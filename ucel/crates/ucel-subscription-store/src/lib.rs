use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubscriptionState {
    Pending,
    Inflight,
    Active,
    Deadletter,
}

impl SubscriptionState {
    fn as_str(self) -> &'static str {
        match self {
            SubscriptionState::Pending => "pending",
            SubscriptionState::Inflight => "inflight",
            SubscriptionState::Active => "active",
            SubscriptionState::Deadletter => "deadletter",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SubscriptionRow {
    pub key: String,
    pub exchange_id: String,
    pub op_id: String,
    pub symbol: Option<String>,
    pub params_json: String,
    pub assigned_conn: Option<String>,
}

pub struct SubscriptionStore {
    conn: Connection,
}

impl SubscriptionStore {
    pub fn open(path: &str) -> Result<Self, String> {
        let conn = if path == ":memory:" {
            Connection::open_in_memory().map_err(|e| e.to_string())?
        } else {
            Connection::open(path).map_err(|e| e.to_string())?
        };
        let this = Self { conn };
        this.init_schema()?;
        Ok(this)
    }

    fn init_schema(&self) -> Result<(), String> {
        self.conn
            .execute_batch(
                "
            CREATE TABLE IF NOT EXISTS subscriptions (
              key TEXT PRIMARY KEY,
              exchange_id TEXT NOT NULL,
              op_id TEXT NOT NULL,
              symbol TEXT,
              params_json TEXT NOT NULL,
              state TEXT NOT NULL,
              assigned_conn TEXT,
              attempts INTEGER NOT NULL DEFAULT 0,
              last_error TEXT,
              updated_at INTEGER NOT NULL,
              first_active_at INTEGER,
              last_message_at INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_subs_exchange_conn_state
              ON subscriptions(exchange_id, assigned_conn, state);
            CREATE INDEX IF NOT EXISTS idx_subs_lookup_fields
              ON subscriptions(exchange_id, assigned_conn, op_id, symbol, params_json);
            ",
            )
            .map_err(|e| e.to_string())
    }

    pub fn seed(&mut self, rows: &[SubscriptionRow], now: i64) -> Result<(), String> {
        let tx = self.conn.transaction().map_err(|e| e.to_string())?;
        for r in rows {
            tx.execute(
                "INSERT INTO subscriptions(key,exchange_id,op_id,symbol,params_json,state,assigned_conn,updated_at)
                 VALUES(?1,?2,?3,?4,?5,'pending',?6,?7)
                 ON CONFLICT(key) DO UPDATE SET
                    exchange_id=excluded.exchange_id,
                    op_id=excluded.op_id,
                    symbol=excluded.symbol,
                    params_json=excluded.params_json,
                    assigned_conn=excluded.assigned_conn,
                    updated_at=excluded.updated_at",
                params![r.key, r.exchange_id, r.op_id, r.symbol, r.params_json, r.assigned_conn, now],
            )
            .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())
    }

    pub fn next_pending_batch(
        &mut self,
        exchange_id: &str,
        conn_id: &str,
        max_n: usize,
        now: i64,
    ) -> Result<Vec<String>, String> {
        let tx = self.conn.transaction().map_err(|e| e.to_string())?;
        let mut stmt = tx
            .prepare(
                "SELECT key FROM subscriptions
             WHERE exchange_id=?1 AND assigned_conn=?2 AND state='pending'
             ORDER BY updated_at ASC LIMIT ?3",
            )
            .map_err(|e| e.to_string())?;

        let mut rows = stmt
            .query(params![exchange_id, conn_id, max_n as i64])
            .map_err(|e| e.to_string())?;
        let mut keys = Vec::new();
        while let Some(row) = rows.next().map_err(|e| e.to_string())? {
            keys.push(row.get::<_, String>(0).map_err(|e| e.to_string())?);
        }
        drop(rows);
        drop(stmt);

        for k in &keys {
            tx.execute(
                "UPDATE subscriptions SET state='inflight', attempts=attempts+1, updated_at=?2
                 WHERE key=?1 AND state='pending'",
                params![k, now],
            )
            .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(keys)
    }

    pub fn find_key_by_fields(
        &self,
        exchange_id: &str,
        conn_id: &str,
        op_id: &str,
        symbol: Option<&str>,
        params_canon: &str,
    ) -> Result<Option<String>, String> {
        self.conn
            .query_row(
                "SELECT key FROM subscriptions
             WHERE exchange_id=?1 AND assigned_conn=?2 AND op_id=?3
               AND ((symbol IS NULL AND ?4 IS NULL) OR symbol=?4)
               AND params_json=?5
             LIMIT 1",
                params![exchange_id, conn_id, op_id, symbol, params_canon],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())
    }

    pub fn mark_active(&self, key: &str, now: i64) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE subscriptions SET state='active', first_active_at=COALESCE(first_active_at, ?2), updated_at=?2 WHERE key=?1",
                params![key, now],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn mark_deadletter(&self, key: &str, reason: &str, now: i64) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE subscriptions SET state='deadletter', last_error=?2, updated_at=?3 WHERE key=?1",
                params![key, reason, now],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn bump_last_message(&self, key: &str, ts: i64) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE subscriptions SET last_message_at=?2, updated_at=?2 WHERE key=?1",
                params![key, ts],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn requeue_active_to_pending(
        &self,
        exchange_id: &str,
        conn_id: &str,
        now: i64,
    ) -> Result<usize, String> {
        self.conn
            .execute(
                "UPDATE subscriptions
             SET state='pending', updated_at=?3
             WHERE exchange_id=?1 AND assigned_conn=?2 AND state IN ('active','inflight')",
                params![exchange_id, conn_id, now],
            )
            .map_err(|e| e.to_string())
    }

    pub fn state_of(&self, key: &str) -> Result<Option<String>, String> {
        self.conn
            .query_row(
                "SELECT state FROM subscriptions WHERE key=?1",
                params![key],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batch_and_lookup_work() {
        let mut store = SubscriptionStore::open(":memory:").unwrap();
        store
            .seed(
                &[SubscriptionRow {
                    key: "x|op|BTC/USDT|{}".into(),
                    exchange_id: "x".into(),
                    op_id: "op".into(),
                    symbol: Some("BTC/USDT".into()),
                    params_json: "{}".into(),
                    assigned_conn: Some("c1".into()),
                }],
                1,
            )
            .unwrap();

        let got = store.next_pending_batch("x", "c1", 10, 2).unwrap();
        assert_eq!(got.len(), 1);
        let k = store
            .find_key_by_fields("x", "c1", "op", Some("BTC/USDT"), "{}")
            .unwrap()
            .unwrap();
        assert_eq!(k, "x|op|BTC/USDT|{}");
    }
}

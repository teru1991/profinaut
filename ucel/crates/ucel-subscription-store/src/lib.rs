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

    pub fn next_pending(&mut self, exchange_id: &str, conn_id: &str, now: i64) -> Result<Option<String>, String> {
        let tx = self.conn.transaction().map_err(|e| e.to_string())?;
        let key: Option<String> = tx
            .query_row(
                "SELECT key FROM subscriptions
                 WHERE exchange_id=?1 AND assigned_conn=?2 AND state='pending'
                 ORDER BY updated_at ASC LIMIT 1",
                params![exchange_id, conn_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        if let Some(ref k) = key {
            tx.execute(
                "UPDATE subscriptions SET state='inflight', attempts=attempts+1, updated_at=?2 WHERE key=?1 AND state='pending'",
                params![k, now],
            )
            .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(key)
    }


    pub fn mark_inflight(&self, key: &str, now: i64) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE subscriptions SET state='inflight', attempts=attempts+1, updated_at=?2 WHERE key=?1",
                params![key, now],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    pub fn mark_active(&self, key: &str, now: i64) -> Result<(), String> {
        self.conn.execute("UPDATE subscriptions SET state='active', first_active_at=COALESCE(first_active_at, ?2), updated_at=?2 WHERE key=?1", params![key, now]).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn mark_deadletter(&self, key: &str, reason: &str, now: i64) -> Result<(), String> {
        self.conn.execute("UPDATE subscriptions SET state='deadletter', last_error=?2, updated_at=?3 WHERE key=?1", params![key, reason, now]).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn bump_last_message(&self, key: &str, ts: i64) -> Result<(), String> {
        self.conn.execute("UPDATE subscriptions SET last_message_at=?2, updated_at=?2 WHERE key=?1", params![key, ts]).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn requeue_active_to_pending(&self, exchange_id: &str, conn_id: &str, now: i64) -> Result<usize, String> {
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
            .query_row("SELECT state FROM subscriptions WHERE key=?1", params![key], |r| r.get(0))
            .optional()
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_transitions_and_reopen_resume() {
        let mut store = SubscriptionStore::open(":memory:").unwrap();
        let row = SubscriptionRow {
            key: "k1".into(), exchange_id: "binance".into(), op_id: "crypto.public.ws.trade".into(),
            symbol: Some("BTC/USDT".into()), params_json: "{}".into(), assigned_conn: Some("c1".into())
        };
        store.seed(&[row], 1).unwrap();
        let next = store.next_pending("binance", "c1", 2).unwrap().unwrap();
        assert_eq!(next, "k1");
        store.mark_active("k1", 3).unwrap();
        assert_eq!(store.state_of("k1").unwrap().unwrap(), SubscriptionState::Active.as_str());
        store.requeue_active_to_pending("binance", "c1", 4).unwrap();
        assert_eq!(store.state_of("k1").unwrap().unwrap(), SubscriptionState::Pending.as_str());
    }
}

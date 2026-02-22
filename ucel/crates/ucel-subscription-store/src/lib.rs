use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

pub struct SubscriptionStore {
    conn: Connection,
}

#[derive(Debug, Clone)]
pub struct SubscriptionRow {
    pub key: String,
    pub exchange_id: String,
    pub op_id: String,
    pub symbol: Option<String>,
    pub params_canon: String,
    pub assigned_conn: String,
}

impl SubscriptionStore {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        let s = Self { conn };
        s.migrate()?;
        Ok(s)
    }

    fn migrate(&self) -> Result<(), String> {
        self.conn
            .execute_batch(
                r#"
CREATE TABLE IF NOT EXISTS subscriptions (
  key TEXT PRIMARY KEY,
  exchange_id TEXT NOT NULL,
  op_id TEXT NOT NULL,
  symbol TEXT,
  params_canon TEXT NOT NULL,
  assigned_conn TEXT NOT NULL,
  state TEXT NOT NULL,
  attempts INTEGER NOT NULL DEFAULT 0,
  last_error TEXT,
  last_message_at INTEGER,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_subs_exchange_conn_state
ON subscriptions(exchange_id, assigned_conn, state);
CREATE INDEX IF NOT EXISTS idx_subs_lookup_fields
ON subscriptions(exchange_id, op_id, symbol, params_canon);
"#,
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn seed(&mut self, rows: &[SubscriptionRow], now: i64) -> Result<(), String> {
        let tx = self.conn.transaction().map_err(|e| e.to_string())?;
        for r in rows {
            tx.execute(
                r#"
INSERT OR IGNORE INTO subscriptions
(key, exchange_id, op_id, symbol, params_canon, assigned_conn, state, attempts, last_error, last_message_at, created_at, updated_at)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'pending', 0, NULL, NULL, ?7, ?7)
"#,
                params![r.key, r.exchange_id, r.op_id, r.symbol, r.params_canon, r.assigned_conn, now],
            ).map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
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
                r#"
SELECT key FROM subscriptions
WHERE exchange_id=?1 AND assigned_conn=?2 AND op_id=?3
  AND ( (symbol IS NULL AND ?4 IS NULL) OR symbol=?4 )
  AND params_canon=?5
LIMIT 1
"#,
                params![exchange_id, conn_id, op_id, symbol, params_canon],
                |r| r.get::<_, String>(0),
            )
            .optional()
            .map_err(|e| e.to_string())
    }

    pub fn next_pending_batch(
        &mut self,
        exchange_id: &str,
        conn_id: &str,
        max_n: usize,
        now: i64,
    ) -> Result<Vec<String>, String> {
        let tx = self.conn.transaction().map_err(|e| e.to_string())?;

        let keys = {
            let mut stmt = tx
                .prepare(
                    r#"
SELECT key FROM subscriptions
WHERE exchange_id=?1 AND assigned_conn=?2 AND state='pending'
ORDER BY updated_at ASC
LIMIT ?3
"#,
                )
                .map_err(|e| e.to_string())?;

            let mut rows = stmt
                .query(params![exchange_id, conn_id, max_n as i64])
                .map_err(|e| e.to_string())?;

            let mut keys = Vec::new();
            while let Some(row) = rows.next().map_err(|e| e.to_string())? {
                keys.push(row.get::<_, String>(0).map_err(|e| e.to_string())?);
            }
            keys
        };

        for k in &keys {
            tx.execute(
                r#"
UPDATE subscriptions
SET state='inflight', attempts=attempts+1, updated_at=?2
WHERE key=?1 AND state='pending'
"#,
                params![k, now],
            )
            .map_err(|e| e.to_string())?;
        }

        tx.commit().map_err(|e| e.to_string())?;
        Ok(keys)
    }

    pub fn mark_active(&mut self, key: &str, now: i64) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE subscriptions SET state='active', updated_at=?2 WHERE key=?1",
                params![key, now],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn bump_last_message(&mut self, key: &str, now: i64) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE subscriptions SET last_message_at=?2, updated_at=?2 WHERE key=?1",
                params![key, now],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn mark_deadletter(&mut self, key: &str, reason: &str, now: i64) -> Result<(), String> {
        self.conn.execute(
            "UPDATE subscriptions SET state='deadletter', last_error=?2, updated_at=?3 WHERE key=?1",
            params![key, reason, now],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn requeue_connection(
        &mut self,
        exchange_id: &str,
        conn_id: &str,
        now: i64,
    ) -> Result<(), String> {
        self.conn
            .execute(
                r#"
UPDATE subscriptions
SET state='pending', updated_at=?3
WHERE exchange_id=?1 AND assigned_conn=?2 AND state IN ('active','inflight')
"#,
                params![exchange_id, conn_id, now],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

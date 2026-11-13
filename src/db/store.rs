//! Owns the single SQLite connection. Only the DB worker thread touches this.
use crate::db::{DbError, schema};
use rusqlite::Connection;
use std::path::Path;

pub struct Store {
    conn: Connection,
}

impl Store {
    /// Open (creating if needed), apply migrations, and enable WAL.
    pub fn open(path: &Path) -> Result<Self, DbError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut conn = Connection::open(path)?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        schema::migrate(&mut conn)?;
        Ok(Self { conn })
    }

    /// Open an in-memory database (used by tests).
    pub fn open_in_memory() -> Result<Self, DbError> {
        let mut conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        schema::migrate(&mut conn)?;
        Ok(Self { conn })
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    /// Flush WAL into the main db file — called on graceful shutdown.
    pub fn checkpoint(&self) -> Result<(), DbError> {
        self.conn
            .pragma_update(None, "wal_checkpoint", "TRUNCATE")?;
        Ok(())
    }
}

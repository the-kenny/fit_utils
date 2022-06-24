use std::path::Path;

use log::*;
use rusqlite::*;
use thiserror::Error;

use crate::wahoo::WahooWebhook;

pub struct Db(rusqlite::Connection);

#[derive(Debug, Error)]
pub enum Error {
    #[error("Sqlite Error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Serde Error: {0}")]
    Serde(#[from] serde_json::Error),
}

impl Db {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Db, Error> {
        let mut db = Connection::open(path)?;
        initialize(&mut db)?;
        Ok(Db(db))
    }

    pub fn in_memory() -> Result<Db, Error> {
        let mut db = Connection::open_in_memory()?;
        initialize(&mut db)?;
        Ok(Db(db))
    }

    pub fn insert_webhook_row(&mut self, row: &WahooWebhook) -> Result<i64, Error> {
        let data = serde_json::to_string(&row)?;
        let created_at = chrono::Utc::now();
        self.0.execute(
            "insert into wahoo_webhooks (data, created_at) values (?1, ?2)",
            params![data, created_at],
        )?;
        Ok(self.0.last_insert_rowid())
    }

    pub fn webhook_rows(&self) -> Result<Vec<WahooWebhook>, Error> {
        let mut stmt = self
            .0
            .prepare("select data from wahoo_webhooks order by created_at desc")?;

        let iter = stmt
            .query_map(params![], |row| row.get(0))?
            .map(|r| r.unwrap())
            .map(|r| serde_json::from_value::<WahooWebhook>(r).unwrap());

        Ok(iter.collect())
    }
}

fn initialize(db: &mut Connection) -> Result<(), Error> {
    let exists = {
        let mut stmt = db.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='wahoo_webhooks'",
        )?;
        let exists = stmt.query(params![])?.next()?.is_some();
        exists
    };

    if exists {
        info!("Schema alread initialized");
        Ok(())
    } else {
        info!("Initializing schema...");
        Ok(db.execute_batch(include_str!("schema.sql"))?)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_initialization() {
        let db = Db::in_memory().unwrap();
        assert_eq!(db.webhook_rows().unwrap(), vec![]);
    }

    #[test]
    fn test_insert_query_roundtrip() {
        let mut db = Db::in_memory().unwrap();

        let wh = serde_json::from_str(include_str!("../test_data/wahoo_webhook.json")).unwrap();

        let result = db.insert_webhook_row(&wh);
        assert!(result.is_ok());

        assert_eq!(db.webhook_rows().unwrap(), vec![wh]);
    }
}

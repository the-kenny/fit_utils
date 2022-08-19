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
    fn new_with_initialize(conn: Connection) -> Result<Self, Error> {
        let mut db = Db(conn);
        db.0.pragma_update(None, "foreign_keys", &"ON")?;
        db.run_migrations()?;
        Ok(db)
    }

    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        info!("Opening database at {}", path.as_ref().display());
        Self::new_with_initialize(Connection::open(path)?)
    }

    pub fn in_memory() -> Result<Self, Error> {
        Self::new_with_initialize(Connection::open_in_memory()?)
    }

    pub fn raw(&mut self) -> &mut Connection {
        &mut self.0
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

    fn run_migrations(&mut self) -> Result<(), rusqlite::Error> {
        for (name, sql) in MIGRATIONS {
            if self.is_new_migration(name)? {
                debug!("Executing migration {}", name);

                self.0.execute_batch(sql)?;

                self.0.execute(
                    "insert into migrations (name, executed) values (?, ?)",
                    params![name, chrono::Local::now()],
                )?;
            } else {
                debug!("Migration {name} already run")
            }
        }

        Ok(())
    }

    fn is_new_migration(&mut self, name: &str) -> Result<bool, rusqlite::Error> {
        let migration_table_exists = self
            .0
            .query_row(
                "SELECT name FROM sqlite_master WHERE type='table' AND name='migrations'",
                [],
                |_| Ok(()),
            )
            .optional()?
            .is_some();

        if migration_table_exists == false {
            Ok(true)
        } else {
            Ok(self
                .0
                .query_row(
                    "select true from migrations where name = ?",
                    params![name],
                    |_| Ok(()),
                )
                .optional()?
                .is_none())
        }
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

macro_rules! migration {
    ( $x:expr ) => {
        ($x, include_str!(concat!("migrations/", $x)))
    };
}
const MIGRATIONS: &[(&str, &str)] = &[
    migration!("000-migrations.sql"),
    migration!("001-wahoo_webhooks.sql"),
    migration!("002-fit_files.sql"),
];

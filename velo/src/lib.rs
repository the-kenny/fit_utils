use std::path::PathBuf;

pub mod sqlite;
pub mod wahoo;

pub struct AppConfig {
    pub wahoo_webhook_token: String,
    pub sqlite_directory: PathBuf,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        use std::env;
        Ok(Self {
            wahoo_webhook_token: env::var("WAHOO_WEBHOOK_TOKEN")?,
            sqlite_directory: env::var("SQLITE_DIRECTORY")?.into(),
        })
    }
}

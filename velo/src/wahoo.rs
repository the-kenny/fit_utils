use std::{collections::HashMap, path::Path};

use log::debug;
use serde::{Deserialize, Serialize};

use crate::sqlite::Db;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WahooWebhook {
    pub event_type: String,
    pub webhook_token: String,
    pub user: WahooUser,
    pub workout_summary: WahooWorkoutSummary,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct WahooUser {
    pub id: u64,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct WahooWorkoutSummary {
    pub id: u64,
    pub file: WahooWorkoutSummaryFile,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct WahooWorkoutSummaryFile {
    pub url: String,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

pub fn handle_webhook(
    sqlite_directory: impl AsRef<Path>,
    webhook: &WahooWebhook,
) -> Result<(), anyhow::Error> {
    let sqlite_filename = format!("wahoo_{}.sqlite", webhook.user.id);
    let mut path = sqlite_directory.as_ref().to_path_buf();
    path.push(sqlite_filename);

    debug!("Opening SQLite database at {}", path.display());
    let mut db = Db::new(&path)?;

    debug!("Inserting webhook {:?} into {}", webhook, path.display());
    let rowid = db.insert_webhook_row(webhook)?;
    debug!("Inserted Webhook {:?} as {}", webhook, rowid);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::WahooWebhook;

    #[test]
    fn test_webhook_deserialization() {
        let input = include_str!("../test_data/wahoo_webhook.json");
        let hook: WahooWebhook = serde_json::from_str(input).unwrap();

        assert_eq!(hook.event_type, "workout_summary");
        assert_eq!(hook.user.id, 60462);
        assert_eq!(hook.webhook_token, "supersecret");
        assert_eq!(
            hook.workout_summary.file.url,
            "https://server.com/4_Mile_Segment_.fit"
        );
    }
}

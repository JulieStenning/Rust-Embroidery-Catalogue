// Disclaimer logic — track and update disclaimer acceptance via the settings table.
// Uses SQLx for database access.

use sqlx::SqliteConnection;
use crate::settings::{get_setting, update_setting};

/// Returns true if the user has already accepted the disclaimer for this installation.
pub async fn is_disclaimer_accepted(conn: &mut SqliteConnection) -> bool {
    match get_setting(conn, "disclaimer_accepted").await {
        Ok(Some(setting)) => setting.value.to_uppercase() == "TRUE",
        _ => false,
    }
}

/// Persists the disclaimer acceptance state in the settings table.
/// Returns true if the update succeeded.
pub async fn set_disclaimer_accepted(conn: &mut SqliteConnection, accepted: bool) -> bool {
    let value = if accepted { "TRUE" } else { "FALSE" };
    update_setting(conn, "disclaimer_accepted", value).await.is_ok()
}

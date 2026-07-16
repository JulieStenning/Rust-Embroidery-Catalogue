// Runtime settings persistence.
// Bootstrap config (environment and startup defaults) lives in `crate::config`.
use crate::database::models::Setting;
use sqlx::SqliteConnection;

/// Load all settings from the database.
pub async fn load_all_settings(conn: &mut SqliteConnection) -> Result<Vec<Setting>, sqlx::Error> {
    sqlx::query_as::<_, Setting>("SELECT key, value, description FROM settings ORDER BY key")
        .fetch_all(conn)
        .await
}

/// Get a single setting by key. Returns None if the key does not exist.
pub async fn get_setting(
    conn: &mut SqliteConnection,
    setting_key: &str,
) -> Result<Option<Setting>, sqlx::Error> {
    sqlx::query_as::<_, Setting>("SELECT key, value, description FROM settings WHERE key = ?")
        .bind(setting_key)
        .fetch_optional(conn)
        .await
}

/// Update the value of an existing setting. Returns the number of rows affected.
pub async fn update_setting(
    conn: &mut SqliteConnection,
    setting_key: &str,
    new_value: &str,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("UPDATE settings SET value = ? WHERE key = ?")
        .bind(new_value)
        .bind(setting_key)
        .execute(conn)
        .await?;
    Ok(result.rows_affected())
}

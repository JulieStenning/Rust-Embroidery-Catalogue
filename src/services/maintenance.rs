use crate::error::AppError;
use crate::settings;
use sqlx::SqliteConnection;

pub const KEY_BACKUP_DATABASE_DESTINATION: &str = "backup.database_destination";
pub const KEY_BACKUP_DESIGNS_DESTINATION: &str = "backup.designs_destination";

pub async fn get_setting_with_default(
    conn: &mut SqliteConnection,
    key: &str,
) -> Result<String, AppError> {
    let current = settings::get_setting(conn, key).await.map_err(|e| AppError::database(e.to_string()))?;
    if let Some(setting) = current {
        return Ok(setting.value);
    }

    let fallback = "".to_string();
    sqlx::query(
        "INSERT INTO settings (key, value, description) VALUES (?, ?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(key)
    .bind(&fallback)
    .bind("Tagging actions default")
    .execute(conn)
    .await
    .map_err(|e| AppError::database(e.to_string()))?;
    Ok(fallback)
}

pub fn is_truthy(raw: &str) -> bool {
    matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_truthy_accepts_expected_variants() {
        assert!(is_truthy("true"));
        assert!(is_truthy("YES"));
        assert!(is_truthy("1"));
        assert!(is_truthy("on"));
        assert!(is_truthy("  true  "));
        assert!(is_truthy("On"));
        assert!(is_truthy("Yes"));

        assert!(!is_truthy("false"));
        assert!(!is_truthy("0"));
        assert!(!is_truthy("no"));
        assert!(!is_truthy("off"));
        assert!(!is_truthy("other"));
    }
}

// Disclaimer logic — track and update disclaimer acceptance via the settings table.
// Uses SQLx for database access.

use crate::error::AppError;
use crate::settings::{get_setting, update_setting};
use sqlx::SqliteConnection;

/// Returns true if the user has already accepted the disclaimer for this installation.
pub async fn is_disclaimer_accepted(conn: &mut SqliteConnection) -> Result<bool, AppError> {
    match get_setting(conn, "disclaimer_accepted").await {
        Ok(Some(setting)) => Ok(setting.value.to_uppercase() == "TRUE"),
        Ok(None) => Ok(false),
        Err(err) => Err(AppError::database(format!("failed to read disclaimer status: {err}"))),
    }
}

/// Persists the disclaimer acceptance state in the settings table.
/// Returns true if the update succeeded.
pub async fn set_disclaimer_accepted(
    conn: &mut SqliteConnection,
    accepted: bool,
) -> Result<bool, AppError> {
    let value = if accepted { "TRUE" } else { "FALSE" };
    update_setting(conn, "disclaimer_accepted", value)
        .await
        .map(|_| true)
        .map_err(|err| AppError::database(format!("failed to persist disclaimer status: {err}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::{Connection, SqliteConnection};

    /// Creates an in-memory SQLite database with the settings table,
    /// ready for testing disclaimer acceptance logic.
    async fn setup_test_conn() -> SqliteConnection {
        let mut conn = SqliteConnection::connect(":memory:")
            .await
            .expect("failed to create in-memory SQLite connection");

        sqlx::query(
            "CREATE TABLE settings (
                key VARCHAR(100) PRIMARY KEY,
                value TEXT NOT NULL,
                description TEXT
            )",
        )
        .execute(&mut conn)
        .await
        .expect("failed to create settings table");

        conn
    }

    /// Pre-inserts a disclaimer_accepted row, mirroring the migration seed.
    async fn seed_disclaimer(conn: &mut SqliteConnection, value: &str) {
        sqlx::query("INSERT INTO settings (key, value) VALUES ('disclaimer_accepted', ?)")
            .bind(value)
            .execute(conn)
            .await
            .expect("seed failed");
    }

    // -----------------------------------------------------------------------
    // is_disclaimer_accepted
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_is_accepted_returns_false_when_no_row_exists() {
        let mut conn = setup_test_conn().await;
        assert!(!is_disclaimer_accepted(&mut conn).await.unwrap());
    }

    #[tokio::test]
    async fn test_is_accepted_returns_false_when_value_is_false() {
        let mut conn = setup_test_conn().await;
        seed_disclaimer(&mut conn, "FALSE").await;
        assert!(!is_disclaimer_accepted(&mut conn).await.unwrap());
    }

    #[tokio::test]
    async fn test_is_accepted_returns_true_when_value_is_true() {
        let mut conn = setup_test_conn().await;
        seed_disclaimer(&mut conn, "TRUE").await;
        assert!(is_disclaimer_accepted(&mut conn).await.unwrap());
    }

    #[tokio::test]
    async fn test_is_accepted_case_insensitive() {
        let mut conn = setup_test_conn().await;
        seed_disclaimer(&mut conn, "True").await;
        assert!(is_disclaimer_accepted(&mut conn).await.unwrap());
    }

    #[tokio::test]
    async fn test_is_accepted_returns_false_for_arbitrary_text() {
        let mut conn = setup_test_conn().await;
        seed_disclaimer(&mut conn, "maybe").await;
        assert!(!is_disclaimer_accepted(&mut conn).await.unwrap());
    }

    #[tokio::test]
    async fn test_is_accepted_returns_false_for_empty_string() {
        let mut conn = setup_test_conn().await;
        seed_disclaimer(&mut conn, "").await;
        assert!(!is_disclaimer_accepted(&mut conn).await.unwrap());
    }

    // -----------------------------------------------------------------------
    // set_disclaimer_accepted
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_set_accepted_true_inserts_true() {
        let mut conn = setup_test_conn().await;
        seed_disclaimer(&mut conn, "FALSE").await;

        assert!(set_disclaimer_accepted(&mut conn, true).await.unwrap());

        let row: (String,) = sqlx::query_as(
            "SELECT value FROM settings WHERE key = 'disclaimer_accepted'",
        )
        .fetch_one(&mut conn)
        .await
        .expect("select failed");

        assert_eq!(row.0, "TRUE");
    }

    #[tokio::test]
    async fn test_set_accepted_false_inserts_false() {
        let mut conn = setup_test_conn().await;
        seed_disclaimer(&mut conn, "TRUE").await;

        assert!(set_disclaimer_accepted(&mut conn, false).await.unwrap());

        let row: (String,) = sqlx::query_as(
            "SELECT value FROM settings WHERE key = 'disclaimer_accepted'",
        )
        .fetch_one(&mut conn)
        .await
        .expect("select failed");

        assert_eq!(row.0, "FALSE");
    }

    #[tokio::test]
    async fn test_set_accepted_overwrites_previous_value() {
        let mut conn = setup_test_conn().await;
        seed_disclaimer(&mut conn, "FALSE").await;

        // Start with FALSE
        assert!(set_disclaimer_accepted(&mut conn, false).await.unwrap());
        assert!(!is_disclaimer_accepted(&mut conn).await.unwrap());

        // Overwrite to TRUE
        assert!(set_disclaimer_accepted(&mut conn, true).await.unwrap());
        assert!(is_disclaimer_accepted(&mut conn).await.unwrap());
    }

    #[tokio::test]
    async fn test_set_accepted_returns_true_on_success() {
        let mut conn = setup_test_conn().await;
        seed_disclaimer(&mut conn, "FALSE").await;

        assert!(set_disclaimer_accepted(&mut conn, true).await.unwrap());
    }
}

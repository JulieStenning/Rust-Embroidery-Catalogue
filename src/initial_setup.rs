// Initial setup logic — track whether the user has completed or skipped the
// first-run setup wizard (designers/sources onboarding) via the settings table.
// Uses SQLx for database access.

use crate::error::AppError;
use crate::settings::get_setting;
use sqlx::SqliteConnection;

/// Returns true if the user has already completed or skipped the initial setup wizard.
pub async fn is_initial_setup_completed(conn: &mut SqliteConnection) -> Result<bool, AppError> {
    match get_setting(conn, "initial_setup_completed").await {
        Ok(Some(setting)) => Ok(setting.value.to_uppercase() == "TRUE"),
        Ok(None) => Ok(false),
        Err(err) => Err(AppError::database(format!(
            "failed to read initial setup status: {err}"
        ))),
    }
}

/// Persists the initial-setup-completed state in the settings table.
/// Uses an UPSERT so the row is created on first call and updated on
/// subsequent calls (the row is not pre-seeded in the shipped database).
/// Returns true if the update succeeded.
pub async fn set_initial_setup_completed(
    conn: &mut SqliteConnection,
    completed: bool,
) -> Result<bool, AppError> {
    let value = if completed { "TRUE" } else { "FALSE" };
    sqlx::query(
        "INSERT INTO settings (key, value) VALUES (?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind("initial_setup_completed")
    .bind(value)
    .execute(conn)
    .await
    .map(|_| true)
    .map_err(|err| {
        AppError::database(format!("failed to persist initial setup status: {err}"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::{Connection, SqliteConnection};

    /// Creates an in-memory SQLite database with the settings table,
    /// ready for testing initial setup logic.
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

    /// Pre-inserts an initial_setup_completed row.
    async fn seed_setup(conn: &mut SqliteConnection, value: &str) {
        sqlx::query("INSERT INTO settings (key, value) VALUES ('initial_setup_completed', ?)")
            .bind(value)
            .execute(conn)
            .await
            .expect("seed failed");
    }

    // -----------------------------------------------------------------------
    // is_initial_setup_completed
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_is_completed_returns_false_when_no_row_exists() {
        let mut conn = setup_test_conn().await;
        assert!(!is_initial_setup_completed(&mut conn).await.unwrap());
    }

    #[tokio::test]
    async fn test_is_completed_returns_false_when_value_is_false() {
        let mut conn = setup_test_conn().await;
        seed_setup(&mut conn, "FALSE").await;
        assert!(!is_initial_setup_completed(&mut conn).await.unwrap());
    }

    #[tokio::test]
    async fn test_is_completed_returns_true_when_value_is_true() {
        let mut conn = setup_test_conn().await;
        seed_setup(&mut conn, "TRUE").await;
        assert!(is_initial_setup_completed(&mut conn).await.unwrap());
    }

    #[tokio::test]
    async fn test_is_completed_case_insensitive() {
        let mut conn = setup_test_conn().await;
        seed_setup(&mut conn, "True").await;
        assert!(is_initial_setup_completed(&mut conn).await.unwrap());
    }

    #[tokio::test]
    async fn test_is_completed_returns_false_for_arbitrary_text() {
        let mut conn = setup_test_conn().await;
        seed_setup(&mut conn, "maybe").await;
        assert!(!is_initial_setup_completed(&mut conn).await.unwrap());
    }

    #[tokio::test]
    async fn test_is_completed_returns_false_for_empty_string() {
        let mut conn = setup_test_conn().await;
        seed_setup(&mut conn, "").await;
        assert!(!is_initial_setup_completed(&mut conn).await.unwrap());
    }

    // -----------------------------------------------------------------------
    // set_initial_setup_completed
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_set_completed_true_inserts_true() {
        let mut conn = setup_test_conn().await;
        seed_setup(&mut conn, "FALSE").await;

        assert!(set_initial_setup_completed(&mut conn, true).await.unwrap());

        let row: (String,) = sqlx::query_as(
            "SELECT value FROM settings WHERE key = 'initial_setup_completed'",
        )
        .fetch_one(&mut conn)
        .await
        .expect("select failed");

        assert_eq!(row.0, "TRUE");
    }

    #[tokio::test]
    async fn test_set_completed_false_inserts_false() {
        let mut conn = setup_test_conn().await;
        seed_setup(&mut conn, "TRUE").await;

        assert!(set_initial_setup_completed(&mut conn, false).await.unwrap());

        let row: (String,) = sqlx::query_as(
            "SELECT value FROM settings WHERE key = 'initial_setup_completed'",
        )
        .fetch_one(&mut conn)
        .await
        .expect("select failed");

        assert_eq!(row.0, "FALSE");
    }

    #[tokio::test]
    async fn test_set_completed_overwrites_previous_value() {
        let mut conn = setup_test_conn().await;
        seed_setup(&mut conn, "FALSE").await;

        // Start with FALSE
        assert!(set_initial_setup_completed(&mut conn, false).await.unwrap());
        assert!(!is_initial_setup_completed(&mut conn).await.unwrap());

        // Overwrite to TRUE
        assert!(set_initial_setup_completed(&mut conn, true).await.unwrap());
        assert!(is_initial_setup_completed(&mut conn).await.unwrap());
    }

    #[tokio::test]
    async fn test_set_completed_creates_row_when_missing() {
        // No pre-seed — the UPSERT must INSERT the row on first call.
        let mut conn = setup_test_conn().await;

        assert!(!is_initial_setup_completed(&mut conn).await.unwrap());

        assert!(set_initial_setup_completed(&mut conn, true).await.unwrap());

        assert!(is_initial_setup_completed(&mut conn).await.unwrap());

        let row: (String,) = sqlx::query_as(
            "SELECT value FROM settings WHERE key = 'initial_setup_completed'",
        )
        .fetch_one(&mut conn)
        .await
        .expect("select failed");

        assert_eq!(row.0, "TRUE");
    }

    #[tokio::test]
    async fn test_set_completed_returns_true_on_success() {
        let mut conn = setup_test_conn().await;
        seed_setup(&mut conn, "FALSE").await;

        assert!(set_initial_setup_completed(&mut conn, true).await.unwrap());
    }
}

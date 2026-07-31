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

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_db() -> sqlx::SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(":memory:")
            .await
            .expect("failed to connect to in-memory SQLite");

        crate::database::migrations::run_migrations(&pool)
            .await
            .expect("failed to run migrations");

        pool
    }

    #[tokio::test]
    async fn test_load_all_settings() {
        let pool = setup_db().await;
        let mut conn = pool.acquire().await.unwrap();

        // Clear settings table
        sqlx::query("DELETE FROM settings").execute(&mut *conn).await.unwrap();

        // 1. Empty state
        let settings = load_all_settings(&mut *conn).await.unwrap();
        assert!(settings.is_empty());

        // 2. Insert out of order and verify they are ordered by key
        sqlx::query("INSERT INTO settings (key, value, description) VALUES (?, ?, ?)")
            .bind("b_key")
            .bind("val_b")
            .bind("desc_b")
            .execute(&mut *conn)
            .await
            .unwrap();

        sqlx::query("INSERT INTO settings (key, value, description) VALUES (?, ?, ?)")
            .bind("a_key")
            .bind("val_a")
            .bind("desc_a")
            .execute(&mut *conn)
            .await
            .unwrap();

        let settings = load_all_settings(&mut *conn).await.unwrap();
        assert_eq!(settings.len(), 2);
        assert_eq!(settings[0].key.as_deref(), Some("a_key"));
        assert_eq!(settings[0].value, "val_a");
        assert_eq!(settings[0].description.as_deref(), Some("desc_a"));
        assert_eq!(settings[1].key.as_deref(), Some("b_key"));
        assert_eq!(settings[1].value, "val_b");
        assert_eq!(settings[1].description.as_deref(), Some("desc_b"));

        // 3. Error path: drop table and verify it returns Err
        sqlx::query("DROP TABLE settings").execute(&mut *conn).await.unwrap();
        let err = load_all_settings(&mut *conn).await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn test_get_setting() {
        let pool = setup_db().await;
        let mut conn = pool.acquire().await.unwrap();

        sqlx::query("DELETE FROM settings").execute(&mut *conn).await.unwrap();

        // 1. Get non-existent key returns None
        let res = get_setting(&mut *conn, "nonexistent").await.unwrap();
        assert!(res.is_none());

        // 2. Get existent key returns Some
        sqlx::query("INSERT INTO settings (key, value, description) VALUES (?, ?, ?)")
            .bind("my_key")
            .bind("my_value")
            .bind("my_desc")
            .execute(&mut *conn)
            .await
            .unwrap();

        let res = get_setting(&mut *conn, "my_key").await.unwrap().unwrap();
        assert_eq!(res.key.as_deref(), Some("my_key"));
        assert_eq!(res.value, "my_value");
        assert_eq!(res.description.as_deref(), Some("my_desc"));

        // 3. Error path: drop table and verify it returns Err
        sqlx::query("DROP TABLE settings").execute(&mut *conn).await.unwrap();
        let err = get_setting(&mut *conn, "my_key").await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn test_update_setting() {
        let pool = setup_db().await;
        let mut conn = pool.acquire().await.unwrap();

        sqlx::query("DELETE FROM settings").execute(&mut *conn).await.unwrap();

        // 1. Update non-existent key returns 0 rows affected
        let rows = update_setting(&mut *conn, "nonexistent", "new_val").await.unwrap();
        assert_eq!(rows, 0);

        // 2. Update existent key returns 1 row affected and modifies the value
        sqlx::query("INSERT INTO settings (key, value, description) VALUES (?, ?, ?)")
            .bind("my_key")
            .bind("my_value")
            .bind("my_desc")
            .execute(&mut *conn)
            .await
            .unwrap();

        let rows = update_setting(&mut *conn, "my_key", "new_val").await.unwrap();
        assert_eq!(rows, 1);

        let res = get_setting(&mut *conn, "my_key").await.unwrap().unwrap();
        assert_eq!(res.value, "new_val");

        // 3. Error path: drop table and verify it returns Err
        sqlx::query("DROP TABLE settings").execute(&mut *conn).await.unwrap();
        let err = update_setting(&mut *conn, "my_key", "new_val").await;
        assert!(err.is_err());
    }
}


// Database migration helpers using SQLx
use sqlx::SqlitePool;

/// Run all pending migrations from the `migrations/` directory.
/// SQLx will track which migrations have been applied via the `_sqlx_migrations` table.
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}

use sqlx::{postgres::PgPoolOptions, PgPool};

pub async fn connect(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}

pub async fn status(pool: &PgPool) -> &'static str {
    match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(pool)
        .await
    {
        Ok(_) => "ok",
        Err(error) => {
            tracing::warn!(%error, "database health check failed");
            "unavailable"
        }
    }
}


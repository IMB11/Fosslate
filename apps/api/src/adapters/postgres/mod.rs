use sqlx::{PgPool, Postgres, Transaction};

pub mod approvals;
pub mod auth;
pub mod current_translations;
pub mod languages;
pub mod namespaces;
pub mod projects;
pub mod setup;
pub mod source_strings;
pub mod stats;
pub mod translations;
pub mod users;
pub mod votes;

#[derive(Clone)]
pub struct PostgresAdapter {
    pool: PgPool,
}

impl PostgresAdapter {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn begin(&self) -> Result<Transaction<'_, Postgres>, sqlx::Error> {
        self.pool.begin().await
    }
}

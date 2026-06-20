use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct User {
    /// Internal numeric user ID used as author, voter, and reviewer references.
    pub id: i64,
    /// Unique username.
    pub username: String,
    /// Time the user was created.
    pub created_at: DateTime<Utc>,
    /// Time the user was last updated.
    pub updated_at: DateTime<Utc>,
}

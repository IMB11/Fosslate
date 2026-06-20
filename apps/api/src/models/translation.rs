use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct Translation {
    pub id: i64,
    pub project_id: i64,
    pub namespace_id: i64,
    pub string_id: i64,
    pub target_language_id: i64,
    pub author_user_id: i64,
    pub value: String,
    pub rating_score: i32,
    pub created_at: DateTime<Utc>,
}

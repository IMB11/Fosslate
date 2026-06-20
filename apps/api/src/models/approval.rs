use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct TranslationApproval {
    pub string_id: i64,
    pub target_language_id: i64,
    pub translation_id: i64,
    pub approved_by_user_id: i64,
    pub approved_at: DateTime<Utc>,
}

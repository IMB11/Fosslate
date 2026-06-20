use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct NamespaceLanguageStats {
    pub project_id: i64,
    pub namespace_id: i64,
    pub target_language_id: i64,
    pub string_count: i32,
    pub translated_count: i32,
    pub approved_count: i32,
    pub candidate_count: i32,
    pub missing_count: i32,
    pub updated_at: DateTime<Utc>,
}

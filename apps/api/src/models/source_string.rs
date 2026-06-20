use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct SourceString {
    /// Internal numeric source string ID used by string-scoped routes.
    pub id: i64,
    /// Internal numeric project ID that owns this string.
    pub project_id: i64,
    /// Internal numeric namespace ID that contains this string.
    pub namespace_id: i64,
    /// Stable source string key unique within the namespace.
    pub identifier: String,
    /// Source-language text.
    pub value: String,
    /// Time the source string was created.
    pub created_at: DateTime<Utc>,
    /// Time the source string was last updated.
    pub updated_at: DateTime<Utc>,
}

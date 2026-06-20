use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct Translation {
    /// Internal numeric translation candidate ID used by vote, approval, and delete routes.
    pub id: i64,
    /// Internal numeric project ID that owns this translation candidate.
    pub project_id: i64,
    /// Internal numeric namespace ID that contains the source string.
    pub namespace_id: i64,
    /// Internal numeric source string ID being translated.
    pub string_id: i64,
    /// Internal numeric target language ID for this candidate.
    pub target_language_id: i64,
    /// User ID of the author that submitted this candidate.
    pub author_user_id: i64,
    /// Candidate translation text in the target language.
    pub value: String,
    /// Aggregate vote score. Higher scores rank earlier when listing candidates.
    pub rating_score: i32,
    /// Time the translation candidate was created.
    pub created_at: DateTime<Utc>,
}

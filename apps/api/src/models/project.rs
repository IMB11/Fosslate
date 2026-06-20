use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::models::Language;

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct Project {
    /// Internal numeric project ID used by related records.
    pub id: i64,
    /// Public UUID used in client-facing project URLs.
    pub public_id: Uuid,
    /// Human-readable project name.
    pub name: String,
    /// Optional asset ID for the project icon.
    pub icon_asset_id: Option<i64>,
    /// Source language that original strings are authored in.
    pub source_language: Language,
    /// Time the project was created.
    pub created_at: DateTime<Utc>,
    /// Time the project was last updated.
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct ProjectTargetLanguage {
    /// Internal numeric target language ID used by language-scoped routes.
    pub id: i64,
    /// Internal numeric project ID that owns this target language.
    pub project_id: i64,
    /// Target language key and display name.
    pub language: Language,
    /// Time the target language was added.
    pub created_at: DateTime<Utc>,
    /// Time the target language was last updated.
    pub updated_at: DateTime<Utc>,
}

use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::models::Language;

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct Project {
    pub id: i64,
    pub public_id: Uuid,
    pub name: String,
    pub icon_asset_id: Option<i64>,
    pub source_language: Language,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct ProjectTargetLanguage {
    pub id: i64,
    pub project_id: i64,
    pub language: Language,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

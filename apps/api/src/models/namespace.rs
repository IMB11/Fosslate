use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct Namespace {
    /// Internal numeric namespace ID used by namespace-scoped routes.
    pub id: i64,
    /// Internal numeric project ID that owns this namespace.
    pub project_id: i64,
    /// Namespace name unique within the active project.
    pub name: String,
    /// Time the namespace was created.
    pub created_at: DateTime<Utc>,
    /// Time the namespace was last updated.
    pub updated_at: DateTime<Utc>,
}

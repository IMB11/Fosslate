use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct AuthUser {
    pub id: i64,
    pub email: String,
    pub username: String,
    pub avatar_url: Option<String>,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct AuthUserResponse {
    pub user: AuthUser,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct AuthProvidersResponse {
    pub password: bool,
    pub sso: SsoProviders,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct SsoProviders {
    pub github: SsoProviderAvailability,
    pub gitlab: SsoProviderAvailability,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct SsoProviderAvailability {
    pub enabled: bool,
    pub start_url: Option<String>,
    pub base_url: Option<String>,
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct AuthUser {
    pub id: i64,
    pub email: String,
    pub username: String,
    pub is_admin: bool,
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

#[derive(Debug, Clone, Copy, Deserialize, Serialize, sqlx::Type, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum AccountSsoProvider {
    Github,
    Gitlab,
}

impl AccountSsoProvider {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Github => "github",
            Self::Gitlab => "gitlab",
        }
    }
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct AccountIdentity {
    pub provider: AccountSsoProvider,
    pub email: Option<String>,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub connected_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct AccountSecurityResponse {
    pub password_enabled: bool,
    pub identities: Vec<AccountIdentity>,
}

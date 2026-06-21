use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SetupStep {
    Github,
    Gitlab,
    Email,
    Complete,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct SetupStatus {
    /// Whether first-time setup still needs to be completed.
    pub required: bool,
    /// Whether the instance setup has been completed.
    pub completed: bool,
    /// Next setup step the UI should show.
    pub next_step: Option<SetupStep>,
    pub github: AuthProviderSetupStatus,
    pub gitlab: AuthProviderSetupStatus,
    pub email: EmailDeliverySetupStatus,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct AuthProviderSetupStatus {
    pub enabled: bool,
    pub configured: bool,
    pub skipped: bool,
    pub has_client_secret: bool,
    pub client_id: Option<String>,
    pub base_url: Option<String>,
    pub scopes: Vec<String>,
    pub callback_url: String,
    pub configured_at: Option<DateTime<Utc>>,
    pub skipped_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct EmailDeliverySetupStatus {
    pub configured: bool,
    pub has_api_key: bool,
    pub provider: Option<String>,
    pub from_name: Option<String>,
    pub from_email: Option<String>,
    pub last_test_recipient: Option<String>,
    pub last_tested_at: Option<DateTime<Utc>>,
    pub last_test_message_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct SetupCompleteResponse {
    pub next: String,
}

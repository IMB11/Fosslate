use crate::{
    adapters::postgres::{
        PostgresAdapter,
        setup::{AuthProviderConfigRow, EmailDeliveryConfigRow},
    },
    error::AppResult,
    models::{AuthProviderSetupStatus, EmailDeliverySetupStatus, SetupStatus, SetupStep},
};

pub const GITHUB_PROVIDER: &str = "github";
pub const GITLAB_PROVIDER: &str = "gitlab";
pub const GITHUB_SCOPES: [&str; 2] = ["read:user", "user:email"];
pub const GITLAB_SCOPES: [&str; 3] = ["openid", "profile", "email"];
pub const DEFAULT_GITLAB_BASE_URL: &str = "https://gitlab.com";

#[derive(Debug, Clone)]
pub struct MaskedInstanceConfig {
    pub completed: bool,
    pub github: AuthProviderSetupStatus,
    pub gitlab: AuthProviderSetupStatus,
    pub email: EmailDeliverySetupStatus,
}

impl MaskedInstanceConfig {
    pub fn into_setup_status(self) -> SetupStatus {
        let next_step = next_step(self.completed, &self.github, &self.gitlab, &self.email);

        SetupStatus {
            required: !self.completed,
            completed: self.completed,
            next_step,
            github: self.github,
            gitlab: self.gitlab,
            email: self.email,
        }
    }
}

pub async fn load_masked_instance_config(
    postgres: &PostgresAdapter,
    public_app_url: &str,
) -> AppResult<MaskedInstanceConfig> {
    let completed = postgres.setup_completed().await?;
    let github = postgres.get_auth_provider_config(GITHUB_PROVIDER).await?;
    let gitlab = postgres.get_auth_provider_config(GITLAB_PROVIDER).await?;
    let email = postgres.get_email_delivery_config().await?;

    Ok(MaskedInstanceConfig {
        completed,
        github: auth_status(GITHUB_PROVIDER, public_app_url, github, &GITHUB_SCOPES),
        gitlab: auth_status(GITLAB_PROVIDER, public_app_url, gitlab, &GITLAB_SCOPES),
        email: email_status(email),
    })
}

pub fn provider_step_done(status: &AuthProviderSetupStatus) -> bool {
    status.configured || status.skipped
}

pub fn scopes(values: &[&str]) -> Vec<String> {
    values.iter().map(|scope| (*scope).to_owned()).collect()
}

fn auth_status(
    provider: &str,
    public_app_url: &str,
    row: Option<AuthProviderConfigRow>,
    default_scopes: &[&str],
) -> AuthProviderSetupStatus {
    let callback_url = format!("{public_app_url}/api/v1/auth/sso/{provider}/callback");

    match row {
        Some(row) => {
            let has_client_secret = row.client_secret_ciphertext.is_some();
            let configured = row.enabled
                && row.configured_at.is_some()
                && row.client_id.is_some()
                && has_client_secret;

            AuthProviderSetupStatus {
                enabled: row.enabled,
                configured,
                skipped: row.skipped_at.is_some() && !row.enabled,
                has_client_secret,
                client_id: row.client_id,
                base_url: row.base_url,
                scopes: row.scopes,
                callback_url,
                configured_at: row.configured_at,
                skipped_at: row.skipped_at,
            }
        }
        None => AuthProviderSetupStatus {
            enabled: false,
            configured: false,
            skipped: false,
            has_client_secret: false,
            client_id: None,
            base_url: if provider == GITLAB_PROVIDER {
                Some(DEFAULT_GITLAB_BASE_URL.to_owned())
            } else {
                None
            },
            scopes: scopes(default_scopes),
            callback_url,
            configured_at: None,
            skipped_at: None,
        },
    }
}

fn email_status(row: Option<EmailDeliveryConfigRow>) -> EmailDeliverySetupStatus {
    match row {
        Some(row) => EmailDeliverySetupStatus {
            configured: true,
            has_api_key: true,
            provider: Some(row.provider),
            from_name: Some(row.from_name),
            from_email: Some(row.from_email),
            last_test_recipient: Some(row.last_test_recipient),
            last_tested_at: Some(row.last_tested_at),
            last_test_message_id: Some(row.last_test_message_id),
        },
        None => EmailDeliverySetupStatus {
            configured: false,
            has_api_key: false,
            provider: None,
            from_name: None,
            from_email: None,
            last_test_recipient: None,
            last_tested_at: None,
            last_test_message_id: None,
        },
    }
}

fn next_step(
    completed: bool,
    github: &AuthProviderSetupStatus,
    gitlab: &AuthProviderSetupStatus,
    email: &EmailDeliverySetupStatus,
) -> Option<SetupStep> {
    if completed {
        None
    } else if !provider_step_done(github) {
        Some(SetupStep::Github)
    } else if !provider_step_done(gitlab) {
        Some(SetupStep::Gitlab)
    } else if !email.configured {
        Some(SetupStep::Email)
    } else {
        Some(SetupStep::Complete)
    }
}

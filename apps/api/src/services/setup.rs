use crate::{
    adapters::{
        postgres::{
            PostgresAdapter,
            setup::{AuthProviderConfigRow, EmailDeliveryConfigRow},
        },
        resend::{EmailDeliveryClient, ResendEmailRequest},
    },
    error::{AppError, AppResult},
    models::{
        AuthProviderSetupStatus, EmailDeliverySetupStatus, SetupCompleteResponse, SetupStatus,
        SetupStep,
    },
};

const GITHUB_PROVIDER: &str = "github";
const GITLAB_PROVIDER: &str = "gitlab";
const GITHUB_SCOPES: [&str; 2] = ["read:user", "user:email"];
const GITLAB_SCOPES: [&str; 3] = ["openid", "profile", "email"];
const DEFAULT_GITLAB_BASE_URL: &str = "https://gitlab.com";

#[derive(Debug, Clone)]
pub struct SaveSsoProvider {
    pub enabled: bool,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub base_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TestEmailDelivery {
    pub resend_api_key: String,
    pub from_name: String,
    pub from_email: String,
    pub test_recipient: String,
}

#[derive(Clone)]
pub struct SetupService {
    postgres: PostgresAdapter,
    email_delivery: EmailDeliveryClient,
    setup_secret: String,
    public_app_url: String,
    secrets_key: String,
}

impl SetupService {
    pub fn new(
        postgres: PostgresAdapter,
        email_delivery: EmailDeliveryClient,
        setup_secret: String,
        public_app_url: String,
        secrets_key: String,
    ) -> Self {
        Self {
            postgres,
            email_delivery,
            setup_secret,
            public_app_url: public_app_url.trim_end_matches('/').to_owned(),
            secrets_key,
        }
    }

    pub async fn verify(&self, authorization: Option<&str>) -> AppResult<SetupStatus> {
        self.ensure_authorized_and_incomplete(authorization).await?;
        self.status_unchecked().await
    }

    pub async fn status(&self, authorization: Option<&str>) -> AppResult<SetupStatus> {
        self.ensure_authorized_and_incomplete(authorization).await?;
        self.status_unchecked().await
    }

    pub async fn setup_required(&self) -> AppResult<bool> {
        Ok(!self.postgres.setup_completed().await?)
    }

    pub async fn save_github_sso(
        &self,
        authorization: Option<&str>,
        input: SaveSsoProvider,
    ) -> AppResult<SetupStatus> {
        self.ensure_authorized_and_incomplete(authorization).await?;

        if input.enabled {
            let client_id = required(input.client_id.as_deref(), "client_id is required")?;
            let client_secret =
                required(input.client_secret.as_deref(), "client_secret is required")?;
            self.postgres
                .configure_auth_provider(
                    GITHUB_PROVIDER,
                    None,
                    client_id,
                    client_secret,
                    &scopes(&GITHUB_SCOPES),
                    &self.secrets_key,
                )
                .await?;
        } else {
            self.postgres
                .skip_auth_provider(GITHUB_PROVIDER, &scopes(&GITHUB_SCOPES))
                .await?;
        }

        self.status_unchecked().await
    }

    pub async fn save_gitlab_sso(
        &self,
        authorization: Option<&str>,
        input: SaveSsoProvider,
    ) -> AppResult<SetupStatus> {
        self.ensure_authorized_and_incomplete(authorization).await?;

        let status = self.status_unchecked().await?;
        if !provider_step_done(&status.github) {
            return Err(AppError::BadRequest("github setup must be completed first"));
        }

        if input.enabled {
            let client_id = required(input.client_id.as_deref(), "client_id is required")?;
            let client_secret =
                required(input.client_secret.as_deref(), "client_secret is required")?;
            let base_url = input
                .base_url
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or(DEFAULT_GITLAB_BASE_URL)
                .trim_end_matches('/')
                .to_owned();

            self.postgres
                .configure_auth_provider(
                    GITLAB_PROVIDER,
                    Some(&base_url),
                    client_id,
                    client_secret,
                    &scopes(&GITLAB_SCOPES),
                    &self.secrets_key,
                )
                .await?;
        } else {
            self.postgres
                .skip_auth_provider(GITLAB_PROVIDER, &scopes(&GITLAB_SCOPES))
                .await?;
        }

        self.status_unchecked().await
    }

    pub async fn send_test_email(
        &self,
        authorization: Option<&str>,
        input: TestEmailDelivery,
    ) -> AppResult<SetupStatus> {
        self.ensure_authorized_and_incomplete(authorization).await?;

        let status = self.status_unchecked().await?;
        if !provider_step_done(&status.github) {
            return Err(AppError::BadRequest("github setup must be completed first"));
        }
        if !provider_step_done(&status.gitlab) {
            return Err(AppError::BadRequest("gitlab setup must be completed first"));
        }

        let resend_api_key = required(Some(&input.resend_api_key), "resend_api_key is required")?;
        let from_name = required(Some(&input.from_name), "from_name is required")?;
        let from_email = required_email(Some(&input.from_email), "from_email is required")?;
        let test_recipient =
            required_email(Some(&input.test_recipient), "test_recipient is required")?;

        let message_id = self
            .email_delivery
            .send_test_email(ResendEmailRequest {
                api_key: resend_api_key,
                from_name,
                from_email,
                test_recipient,
            })
            .await
            .map_err(|error| {
                tracing::warn!(%error, "email delivery test failed");
                AppError::ExternalService("email_delivery_failed")
            })?;

        self.postgres
            .configure_email_delivery(
                resend_api_key,
                from_name,
                from_email,
                test_recipient,
                &message_id,
                &self.secrets_key,
            )
            .await?;

        self.status_unchecked().await
    }

    pub async fn complete(&self, authorization: Option<&str>) -> AppResult<SetupCompleteResponse> {
        self.ensure_authorized_and_incomplete(authorization).await?;

        let status = self.status_unchecked().await?;
        if !status.email.configured {
            return Err(AppError::BadRequest("email delivery must be tested first"));
        }

        self.postgres.complete_setup().await?;

        Ok(SetupCompleteResponse {
            next: "/".to_owned(),
        })
    }

    async fn ensure_authorized_and_incomplete(&self, authorization: Option<&str>) -> AppResult<()> {
        let Some(secret) = bearer_secret(authorization) else {
            return Err(AppError::Unauthorized);
        };

        if secret.as_bytes() != self.setup_secret.as_bytes() {
            return Err(AppError::Unauthorized);
        }

        if self.postgres.setup_completed().await? {
            return Err(AppError::Conflict("setup_complete"));
        }

        Ok(())
    }

    async fn status_unchecked(&self) -> AppResult<SetupStatus> {
        let completed = self.postgres.setup_completed().await?;
        let github = self
            .postgres
            .get_auth_provider_config(GITHUB_PROVIDER)
            .await?;
        let gitlab = self
            .postgres
            .get_auth_provider_config(GITLAB_PROVIDER)
            .await?;
        let email = self.postgres.get_email_delivery_config().await?;

        let github = self.auth_status(GITHUB_PROVIDER, github, &GITHUB_SCOPES);
        let gitlab = self.auth_status(GITLAB_PROVIDER, gitlab, &GITLAB_SCOPES);
        let email = email_status(email);
        let next_step = next_step(completed, &github, &gitlab, &email);

        Ok(SetupStatus {
            required: !completed,
            completed,
            next_step,
            github,
            gitlab,
            email,
        })
    }

    fn auth_status(
        &self,
        provider: &str,
        row: Option<AuthProviderConfigRow>,
        default_scopes: &[&str],
    ) -> AuthProviderSetupStatus {
        let callback_url = format!("{}/auth/{provider}/callback", self.public_app_url);

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

fn provider_step_done(status: &AuthProviderSetupStatus) -> bool {
    status.configured || status.skipped
}

fn bearer_secret(authorization: Option<&str>) -> Option<&str> {
    let value = authorization?.trim();
    let (scheme, secret) = value.split_once(' ')?;

    if scheme.eq_ignore_ascii_case("bearer") {
        let secret = secret.trim();
        if secret.is_empty() {
            None
        } else {
            Some(secret)
        }
    } else {
        None
    }
}

fn required<'a>(value: Option<&'a str>, message: &'static str) -> AppResult<&'a str> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(AppError::BadRequest(message))
}

fn required_email<'a>(value: Option<&'a str>, message: &'static str) -> AppResult<&'a str> {
    let value = required(value, message)?;
    if value.contains('@') {
        Ok(value)
    } else {
        Err(AppError::BadRequest("email address is invalid"))
    }
}

fn scopes(scopes: &[&str]) -> Vec<String> {
    scopes.iter().map(|scope| (*scope).to_owned()).collect()
}

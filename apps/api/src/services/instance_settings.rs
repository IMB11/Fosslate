use crate::{
    adapters::{
        postgres::PostgresAdapter,
        resend::{EmailDeliveryClient, ResendEmailRequest},
    },
    error::{AppError, AppResult},
    models::{AuthUser, InstanceSettings},
    services::instance_config::{
        DEFAULT_GITLAB_BASE_URL, GITHUB_PROVIDER, GITHUB_SCOPES, GITLAB_PROVIDER, GITLAB_SCOPES,
        load_masked_instance_config, scopes,
    },
};

#[derive(Debug, Clone)]
pub struct SaveInstanceSsoProvider {
    pub enabled: bool,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub base_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TestInstanceEmailDelivery {
    pub resend_api_key: Option<String>,
    pub from_name: String,
    pub from_email: String,
    pub test_recipient: String,
}

#[derive(Clone)]
pub struct InstanceSettingsService {
    postgres: PostgresAdapter,
    email_delivery: EmailDeliveryClient,
    setup_secret: String,
    public_app_url: String,
    secrets_key: String,
}

impl InstanceSettingsService {
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

    pub async fn claim_admin(
        &self,
        current_user: &AuthUser,
        setup_secret: &str,
    ) -> AppResult<AuthUser> {
        let setup_secret = setup_secret.trim();
        if setup_secret.is_empty() || setup_secret.as_bytes() != self.setup_secret.as_bytes() {
            return Err(AppError::Unauthorized);
        }

        self.postgres.grant_user_admin(current_user.id).await?;
        Ok(self.postgres.auth_user_by_id(current_user.id).await?)
    }

    pub async fn settings(&self) -> AppResult<InstanceSettings> {
        self.current_settings().await
    }

    pub async fn save_sso(
        &self,
        provider: &str,
        input: SaveInstanceSsoProvider,
    ) -> AppResult<InstanceSettings> {
        let provider = normalized_provider(provider)?;
        let provider_scopes = provider_scopes(provider);

        if !input.enabled {
            self.postgres
                .set_auth_provider_enabled(provider, false, &scopes(provider_scopes))
                .await?;
            return self.current_settings().await;
        }

        let client_id = required(input.client_id.as_deref(), "client_id is required")?;
        let client_secret = input
            .client_secret
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let base_url = if provider == GITLAB_PROVIDER {
            Some(
                input
                    .base_url
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .unwrap_or(DEFAULT_GITLAB_BASE_URL)
                    .trim_end_matches('/')
                    .to_owned(),
            )
        } else {
            None
        };

        if let Some(client_secret) = client_secret {
            self.postgres
                .configure_auth_provider(
                    provider,
                    base_url.as_deref(),
                    client_id,
                    client_secret,
                    &scopes(provider_scopes),
                    &self.secrets_key,
                )
                .await?;
        } else {
            let existing = self.postgres.get_auth_provider_config(provider).await?;
            let has_secret = existing
                .as_ref()
                .and_then(|row| row.client_secret_ciphertext.as_ref())
                .is_some();
            if !has_secret {
                return Err(AppError::BadRequest("client_secret is required"));
            }

            self.postgres
                .configure_auth_provider_without_secret_change(
                    provider,
                    base_url.as_deref(),
                    client_id,
                    &scopes(provider_scopes),
                )
                .await?;
        }

        self.current_settings().await
    }

    pub async fn send_test_email(
        &self,
        input: TestInstanceEmailDelivery,
    ) -> AppResult<InstanceSettings> {
        let from_name = required(Some(&input.from_name), "from_name is required")?;
        let from_email = required_email(Some(&input.from_email), "from_email is required")?;
        let test_recipient =
            required_email(Some(&input.test_recipient), "test_recipient is required")?;
        let resend_api_key = input
            .resend_api_key
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned);
        let resend_api_key = match resend_api_key {
            Some(resend_api_key) => resend_api_key,
            None => self
                .postgres
                .get_reset_email_config(&self.secrets_key)
                .await?
                .map(|config| config.api_key)
                .ok_or(AppError::BadRequest("resend_api_key is required"))?,
        };

        let message_id = self
            .email_delivery
            .send_test_email(ResendEmailRequest {
                api_key: &resend_api_key,
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
                &resend_api_key,
                from_name,
                from_email,
                test_recipient,
                &message_id,
                &self.secrets_key,
            )
            .await?;

        self.current_settings().await
    }

    async fn current_settings(&self) -> AppResult<InstanceSettings> {
        let config = load_masked_instance_config(&self.postgres, &self.public_app_url).await?;
        Ok(InstanceSettings {
            github: config.github,
            gitlab: config.gitlab,
            email: config.email,
        })
    }
}

fn normalized_provider(provider: &str) -> AppResult<&str> {
    match provider.trim().to_ascii_lowercase().as_str() {
        GITHUB_PROVIDER => Ok(GITHUB_PROVIDER),
        GITLAB_PROVIDER => Ok(GITLAB_PROVIDER),
        _ => Err(AppError::NotFound("auth_provider")),
    }
}

fn provider_scopes(provider: &str) -> &'static [&'static str] {
    if provider == GITHUB_PROVIDER {
        &GITHUB_SCOPES
    } else {
        &GITLAB_SCOPES
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

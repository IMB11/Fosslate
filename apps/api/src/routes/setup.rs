use axum::{
    Json,
    extract::State,
    http::{HeaderMap, header},
};
use serde::Deserialize;

use crate::{
    app::AppState,
    error::AppResult,
    models::{SetupCompleteResponse, SetupStatus},
    services::setup::{SaveSsoProvider, TestEmailDelivery},
};

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SaveSsoProviderRequest {
    /// Whether this SSO provider should be enabled. Send `false` to skip this setup step.
    pub enabled: bool,
    /// OAuth client ID from GitHub or GitLab. Required when `enabled` is `true`.
    pub client_id: Option<String>,
    /// OAuth client secret from GitHub or GitLab. Required when `enabled` is `true`.
    pub client_secret: Option<String>,
    /// GitLab instance base URL. Only used by GitLab; defaults to `https://gitlab.com`.
    pub base_url: Option<String>,
}

impl From<SaveSsoProviderRequest> for SaveSsoProvider {
    fn from(request: SaveSsoProviderRequest) -> Self {
        Self {
            enabled: request.enabled,
            client_id: request.client_id,
            client_secret: request.client_secret,
            base_url: request.base_url,
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct TestEmailDeliveryRequest {
    /// Resend API key used to send the setup test email.
    pub resend_api_key: String,
    /// Friendly sender name for Fosslate emails.
    pub from_name: String,
    /// Verified sender email address.
    pub from_email: String,
    /// Recipient address for the setup test email.
    pub test_recipient: String,
}

impl From<TestEmailDeliveryRequest> for TestEmailDelivery {
    fn from(request: TestEmailDeliveryRequest) -> Self {
        Self {
            resend_api_key: request.resend_api_key,
            from_name: request.from_name,
            from_email: request.from_email,
            test_recipient: request.test_recipient,
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/setup/verify",
    tag = "setup",
    operation_id = "verify_setup_secret",
    summary = "Verify the first-run setup secret",
    description = "Verifies the secret code printed in Docker/API logs and returns the next incomplete setup step.",
    params(("Authorization" = String, Header, description = "Bearer setup secret code.")),
    responses(
        (status = 200, description = "Setup secret accepted.", body = SetupStatus),
        (status = 401, description = "Setup secret is missing or invalid.", body = crate::error::ErrorBody),
        (status = 409, description = "Setup is already complete.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn verify_setup_secret(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<Json<SetupStatus>> {
    Ok(Json(
        state.services.setup.verify(authorization(&headers)).await?,
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/setup/status",
    tag = "setup",
    operation_id = "get_setup_status",
    summary = "Get setup status",
    description = "Returns configured/skipped setup steps and OAuth callback URLs.",
    params(("Authorization" = String, Header, description = "Bearer setup secret code.")),
    responses(
        (status = 200, description = "Setup status returned.", body = SetupStatus),
        (status = 401, description = "Setup secret is missing or invalid.", body = crate::error::ErrorBody),
        (status = 409, description = "Setup is already complete.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn get_setup_status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<Json<SetupStatus>> {
    Ok(Json(
        state.services.setup.status(authorization(&headers)).await?,
    ))
}

#[utoipa::path(
    put,
    path = "/api/v1/setup/sso/github",
    tag = "setup",
    operation_id = "save_github_sso_setup",
    summary = "Configure or skip GitHub SSO",
    description = "Stores encrypted GitHub OAuth credentials, or marks GitHub SSO as skipped.",
    params(("Authorization" = String, Header, description = "Bearer setup secret code.")),
    request_body(content = SaveSsoProviderRequest, description = "GitHub SSO setup choice."),
    responses(
        (status = 200, description = "GitHub SSO step saved.", body = SetupStatus),
        (status = 400, description = "GitHub SSO request is invalid.", body = crate::error::ErrorBody),
        (status = 401, description = "Setup secret is missing or invalid.", body = crate::error::ErrorBody),
        (status = 409, description = "Setup is already complete.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn save_github_sso_setup(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<SaveSsoProviderRequest>,
) -> AppResult<Json<SetupStatus>> {
    Ok(Json(
        state
            .services
            .setup
            .save_github_sso(authorization(&headers), request.into())
            .await?,
    ))
}

#[utoipa::path(
    put,
    path = "/api/v1/setup/sso/gitlab",
    tag = "setup",
    operation_id = "save_gitlab_sso_setup",
    summary = "Configure or skip GitLab SSO",
    description = "Stores encrypted GitLab OAuth credentials, or marks GitLab SSO as skipped.",
    params(("Authorization" = String, Header, description = "Bearer setup secret code.")),
    request_body(content = SaveSsoProviderRequest, description = "GitLab SSO setup choice."),
    responses(
        (status = 200, description = "GitLab SSO step saved.", body = SetupStatus),
        (status = 400, description = "GitLab SSO request is invalid or GitHub setup is not complete.", body = crate::error::ErrorBody),
        (status = 401, description = "Setup secret is missing or invalid.", body = crate::error::ErrorBody),
        (status = 409, description = "Setup is already complete.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn save_gitlab_sso_setup(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<SaveSsoProviderRequest>,
) -> AppResult<Json<SetupStatus>> {
    Ok(Json(
        state
            .services
            .setup
            .save_gitlab_sso(authorization(&headers), request.into())
            .await?,
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/setup/email/test",
    tag = "setup",
    operation_id = "test_email_delivery_setup",
    summary = "Send a Resend setup test email",
    description = "Sends a test email through Resend and stores encrypted email delivery settings when the send succeeds.",
    params(("Authorization" = String, Header, description = "Bearer setup secret code.")),
    request_body(content = TestEmailDeliveryRequest, description = "Resend test email settings."),
    responses(
        (status = 200, description = "Test email sent and email delivery configured.", body = SetupStatus),
        (status = 400, description = "Email setup request is invalid or prior setup steps are incomplete.", body = crate::error::ErrorBody),
        (status = 401, description = "Setup secret is missing or invalid.", body = crate::error::ErrorBody),
        (status = 409, description = "Setup is already complete.", body = crate::error::ErrorBody),
        (status = 502, description = "Resend rejected or failed the test email request.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn test_email_delivery_setup(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<TestEmailDeliveryRequest>,
) -> AppResult<Json<SetupStatus>> {
    Ok(Json(
        state
            .services
            .setup
            .send_test_email(authorization(&headers), request.into())
            .await?,
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/setup/complete",
    tag = "setup",
    operation_id = "complete_setup",
    summary = "Complete first-run setup",
    description = "Marks setup complete after the email delivery step has passed.",
    params(("Authorization" = String, Header, description = "Bearer setup secret code.")),
    responses(
        (status = 200, description = "Setup completed.", body = SetupCompleteResponse),
        (status = 400, description = "Email delivery has not been tested yet.", body = crate::error::ErrorBody),
        (status = 401, description = "Setup secret is missing or invalid.", body = crate::error::ErrorBody),
        (status = 409, description = "Setup is already complete.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn complete_setup(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<Json<SetupCompleteResponse>> {
    Ok(Json(
        state
            .services
            .setup
            .complete(authorization(&headers))
            .await?,
    ))
}

fn authorization(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
}

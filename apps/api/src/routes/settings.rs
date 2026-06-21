use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::Redirect,
};
use serde::Deserialize;

use crate::{
    app::AppState,
    error::{AppError, AppResult},
    models::{AccountSecurityResponse, AuthUser, AuthUserResponse, InstanceSettings},
    routes::auth::CurrentUser,
    services::{
        auth::UpdateAccountPasswordInput,
        instance_settings::{SaveInstanceSsoProvider, TestInstanceEmailDelivery},
    },
};

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ClaimInstanceAdminRequest {
    pub setup_secret: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateAccountPasswordRequest {
    pub password: String,
    pub password_confirmation: String,
}

impl From<UpdateAccountPasswordRequest> for UpdateAccountPasswordInput {
    fn from(request: UpdateAccountPasswordRequest) -> Self {
        Self {
            password: request.password,
            password_confirmation: request.password_confirmation,
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SaveInstanceSsoProviderRequest {
    pub enabled: bool,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub base_url: Option<String>,
}

impl From<SaveInstanceSsoProviderRequest> for SaveInstanceSsoProvider {
    fn from(request: SaveInstanceSsoProviderRequest) -> Self {
        Self {
            enabled: request.enabled,
            client_id: request.client_id,
            client_secret: request.client_secret,
            base_url: request.base_url,
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct TestInstanceEmailDeliveryRequest {
    pub resend_api_key: Option<String>,
    pub from_name: String,
    pub from_email: String,
    pub test_recipient: String,
}

impl From<TestInstanceEmailDeliveryRequest> for TestInstanceEmailDelivery {
    fn from(request: TestInstanceEmailDeliveryRequest) -> Self {
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
    path = "/api/v1/settings/instance/admin/claim",
    tag = "settings",
    operation_id = "claim_instance_admin",
    summary = "Claim instance admin role",
    description = "Grants the current authenticated user the global admin role when the submitted code matches the current backend log secret.",
    request_body(content = ClaimInstanceAdminRequest),
    responses(
        (status = 200, description = "Admin role granted.", body = AuthUserResponse),
        (status = 401, description = "Admin claim secret is invalid.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn claim_instance_admin(
    State(state): State<AppState>,
    CurrentUser(current_user): CurrentUser,
    Json(request): Json<ClaimInstanceAdminRequest>,
) -> AppResult<Json<AuthUserResponse>> {
    let user = state
        .services
        .instance_settings
        .claim_admin(&current_user, &request.setup_secret)
        .await?;
    Ok(Json(AuthUserResponse { user }))
}

#[utoipa::path(
    get,
    path = "/api/v1/settings/profile/security",
    tag = "settings",
    operation_id = "get_account_security",
    summary = "Get current account auth methods",
    description = "Returns whether the current user has a password and which GitHub/GitLab identities are connected.",
    responses(
        (status = 200, description = "Current account auth methods.", body = AccountSecurityResponse),
        (status = 401, description = "No valid session.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn get_account_security(
    State(state): State<AppState>,
    CurrentUser(current_user): CurrentUser,
) -> AppResult<Json<AccountSecurityResponse>> {
    Ok(Json(
        state.services.auth.account_security(&current_user).await?,
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/settings/profile/password",
    tag = "settings",
    operation_id = "update_account_password",
    summary = "Set current account password",
    description = "Sets or replaces the password for the currently authenticated account using the standard password policy.",
    request_body(content = UpdateAccountPasswordRequest),
    responses(
        (status = 200, description = "Updated account auth methods.", body = AccountSecurityResponse),
        (status = 400, description = "Password request is invalid.", body = crate::error::ErrorBody),
        (status = 401, description = "No valid session.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn update_account_password(
    State(state): State<AppState>,
    CurrentUser(current_user): CurrentUser,
    Json(request): Json<UpdateAccountPasswordRequest>,
) -> AppResult<Json<AccountSecurityResponse>> {
    Ok(Json(
        state
            .services
            .auth
            .update_account_password(&current_user, request.into())
            .await?,
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/settings/profile/sso/{provider}/start",
    tag = "settings",
    operation_id = "start_account_sso",
    summary = "Start linking an SSO identity",
    description = "Starts an OAuth flow that links the resolved GitHub/GitLab identity to the currently authenticated account.",
    params(("provider" = String, Path, description = "`github` or `gitlab`.")),
    responses(
        (status = 303, description = "Redirect to provider authorization URL."),
        (status = 401, description = "No valid session.", body = crate::error::ErrorBody),
        (status = 404, description = "Provider is not configured.", body = crate::error::ErrorBody),
        (status = 409, description = "Provider is already linked.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn start_account_sso(
    State(state): State<AppState>,
    CurrentUser(current_user): CurrentUser,
    Path(provider): Path<String>,
) -> AppResult<Redirect> {
    let start = state
        .services
        .auth
        .start_account_sso(&current_user, &provider)
        .await?;
    Ok(Redirect::to(&start.redirect_url))
}

#[utoipa::path(
    delete,
    path = "/api/v1/settings/profile/sso/{provider}",
    tag = "settings",
    operation_id = "remove_account_sso",
    summary = "Remove a linked SSO identity",
    description = "Removes a GitHub/GitLab identity from the current account. If this is the final SSO identity, the account must already have a password.",
    params(("provider" = String, Path, description = "`github` or `gitlab`.")),
    responses(
        (status = 200, description = "Updated account auth methods.", body = AccountSecurityResponse),
        (status = 401, description = "No valid session.", body = crate::error::ErrorBody),
        (status = 409, description = "A password must be added before removing the final SSO identity.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn remove_account_sso(
    State(state): State<AppState>,
    CurrentUser(current_user): CurrentUser,
    Path(provider): Path<String>,
) -> AppResult<(StatusCode, Json<AccountSecurityResponse>)> {
    Ok((
        StatusCode::OK,
        Json(
            state
                .services
                .auth
                .remove_account_sso(&current_user, &provider)
                .await?,
        ),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/settings/instance",
    tag = "settings",
    operation_id = "get_instance_settings",
    summary = "Get masked instance settings",
    description = "Returns masked OAuth and Resend instance settings for admins without exposing stored secrets.",
    responses(
        (status = 200, description = "Masked instance settings.", body = InstanceSettings),
        (status = 403, description = "Current user is not an admin.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn get_instance_settings(
    State(state): State<AppState>,
    CurrentUser(current_user): CurrentUser,
) -> AppResult<Json<InstanceSettings>> {
    ensure_admin(&current_user)?;
    Ok(Json(state.services.instance_settings.settings().await?))
}

#[utoipa::path(
    put,
    path = "/api/v1/settings/instance/sso/{provider}",
    tag = "settings",
    operation_id = "save_instance_sso_provider",
    summary = "Update an instance SSO provider",
    description = "Enables, updates, or disables a GitHub/GitLab OAuth provider. Stored client secrets are retained when the field is omitted.",
    params(("provider" = String, Path, description = "`github` or `gitlab`.")),
    request_body(content = SaveInstanceSsoProviderRequest),
    responses(
        (status = 200, description = "Masked instance settings.", body = InstanceSettings),
        (status = 400, description = "Provider settings are invalid.", body = crate::error::ErrorBody),
        (status = 403, description = "Current user is not an admin.", body = crate::error::ErrorBody),
        (status = 404, description = "Provider was not found.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn save_instance_sso_provider(
    State(state): State<AppState>,
    CurrentUser(current_user): CurrentUser,
    Path(provider): Path<String>,
    Json(request): Json<SaveInstanceSsoProviderRequest>,
) -> AppResult<Json<InstanceSettings>> {
    ensure_admin(&current_user)?;
    Ok(Json(
        state
            .services
            .instance_settings
            .save_sso(&provider, request.into())
            .await?,
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/settings/instance/email/test",
    tag = "settings",
    operation_id = "test_instance_email_delivery",
    summary = "Send an instance email delivery test",
    description = "Sends a Resend test email and stores encrypted email settings on success. The stored API key is retained when the field is omitted.",
    request_body(content = TestInstanceEmailDeliveryRequest),
    responses(
        (status = 200, description = "Masked instance settings.", body = InstanceSettings),
        (status = 400, description = "Email settings are invalid.", body = crate::error::ErrorBody),
        (status = 403, description = "Current user is not an admin.", body = crate::error::ErrorBody),
        (status = 502, description = "Resend rejected or failed the test email request.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn test_instance_email_delivery(
    State(state): State<AppState>,
    CurrentUser(current_user): CurrentUser,
    Json(request): Json<TestInstanceEmailDeliveryRequest>,
) -> AppResult<Json<InstanceSettings>> {
    ensure_admin(&current_user)?;
    Ok(Json(
        state
            .services
            .instance_settings
            .send_test_email(request.into())
            .await?,
    ))
}

fn ensure_admin(user: &AuthUser) -> AppResult<()> {
    if user.is_admin {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

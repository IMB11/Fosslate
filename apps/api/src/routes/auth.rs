use axum::{
    Json,
    body::Body,
    extract::{FromRequestParts, Path, Query, State},
    http::{
        HeaderMap, HeaderValue, Method, Request, StatusCode,
        header::{COOKIE, LOCATION, SET_COOKIE},
        request::Parts,
    },
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;

use crate::{
    app::AppState,
    error::{AppError, AppResult},
    models::{AuthProvidersResponse, AuthUser, AuthUserResponse},
    services::auth::{
        AuthCookies, ForgotPasswordInput, LoginInput, RequestContext, ResetPasswordInput,
        SignupCompleteInput, SignupStartInput,
    },
};

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SignupStartRequest {
    pub email: String,
    pub password: String,
}

impl From<SignupStartRequest> for SignupStartInput {
    fn from(request: SignupStartRequest) -> Self {
        Self {
            email: request.email,
            password: request.password,
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SignupCompleteRequest {
    pub email: String,
    pub password: String,
    pub code: String,
}

impl From<SignupCompleteRequest> for SignupCompleteInput {
    fn from(request: SignupCompleteRequest) -> Self {
        Self {
            email: request.email,
            password: request.password,
            code: request.code,
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

impl From<LoginRequest> for LoginInput {
    fn from(request: LoginRequest) -> Self {
        Self {
            email: request.email,
            password: request.password,
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

impl From<ForgotPasswordRequest> for ForgotPasswordInput {
    fn from(request: ForgotPasswordRequest) -> Self {
        Self {
            email: request.email,
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub password: String,
    pub password_confirmation: String,
}

impl From<ResetPasswordRequest> for ResetPasswordInput {
    fn from(request: ResetPasswordRequest) -> Self {
        Self {
            token: request.token,
            password: request.password,
            password_confirmation: request.password_confirmation,
        }
    }
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct SsoStartQuery {
    pub redirect_to: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct SsoCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CurrentUser(pub AuthUser);

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .map(CurrentUser)
            .ok_or(AppError::Unauthorized)
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/providers",
    tag = "auth",
    operation_id = "get_auth_providers",
    summary = "List available auth providers",
    description = "Returns password availability and configured GitHub/GitLab SSO start URLs without exposing provider secrets.",
    responses(
        (status = 200, description = "Available auth providers.", body = AuthProvidersResponse),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn get_auth_providers(
    State(state): State<AppState>,
) -> AppResult<Json<AuthProvidersResponse>> {
    Ok(Json(state.services.auth.providers().await?))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/signup/start",
    tag = "auth",
    operation_id = "start_signup",
    summary = "Start account creation",
    description = "Validates signup input and sends a one-time email verification code without creating or authenticating a user.",
    request_body(content = SignupStartRequest),
    responses(
        (status = 202, description = "Verification email accepted."),
        (status = 400, description = "Signup request is invalid.", body = crate::error::ErrorBody),
        (status = 409, description = "Account already exists or email delivery is not configured.", body = crate::error::ErrorBody),
        (status = 502, description = "Email delivery failed.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn start_signup(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<SignupStartRequest>,
) -> AppResult<StatusCode> {
    state
        .services
        .auth
        .start_signup(request.into(), request_context(&headers))
        .await?;
    Ok(StatusCode::ACCEPTED)
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/signup/complete",
    tag = "auth",
    operation_id = "complete_signup",
    summary = "Complete account creation",
    description = "Consumes an unexpired one-time email verification code, creates a verified password-backed user, and sets session cookies.",
    request_body(content = SignupCompleteRequest),
    responses(
        (status = 200, description = "Account created and session cookies set.", body = AuthUserResponse),
        (status = 400, description = "Signup request is invalid.", body = crate::error::ErrorBody),
        (status = 401, description = "Verification code is invalid or expired.", body = crate::error::ErrorBody),
        (status = 409, description = "Account already exists.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn complete_signup(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<SignupCompleteRequest>,
) -> AppResult<(HeaderMap, Json<AuthUserResponse>)> {
    let session = state
        .services
        .auth
        .complete_signup(request.into(), request_context(&headers))
        .await?;
    let headers = cookie_headers(&state, &session.cookies)?;
    Ok((headers, Json(AuthUserResponse { user: session.user })))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tag = "auth",
    operation_id = "login",
    summary = "Log in with email and password",
    description = "Verifies email/password credentials, starts a browser session, and sets HttpOnly session cookies plus a CSRF cookie.",
    request_body(content = LoginRequest),
    responses(
        (status = 200, description = "Login accepted and session cookies set.", body = AuthUserResponse),
        (status = 401, description = "Credentials are invalid.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<LoginRequest>,
) -> AppResult<(HeaderMap, Json<AuthUserResponse>)> {
    let session = state
        .services
        .auth
        .login(request.into(), request_context(&headers))
        .await?;
    let headers = cookie_headers(&state, &session.cookies)?;
    Ok((headers, Json(AuthUserResponse { user: session.user })))
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/session",
    tag = "auth",
    operation_id = "get_auth_session",
    summary = "Get current session user",
    description = "Reads the access session cookie and returns the authenticated user when the session is valid.",
    responses(
        (status = 200, description = "Current user.", body = AuthUserResponse),
        (status = 401, description = "No valid session.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn get_auth_session(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<Json<AuthUserResponse>> {
    let session = state
        .services
        .auth
        .current_session(cookie_header(&headers))
        .await?;
    Ok(Json(AuthUserResponse { user: session.user }))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/session/refresh",
    tag = "auth",
    operation_id = "refresh_auth_session",
    summary = "Refresh current session",
    description = "Rotates the refresh session cookie, revokes the previous session row, and returns the authenticated user.",
    responses(
        (status = 200, description = "Session refreshed and cookies rotated.", body = AuthUserResponse),
        (status = 401, description = "Refresh cookie is missing or invalid.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn refresh_auth_session(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<(HeaderMap, Json<AuthUserResponse>)> {
    let session = state
        .services
        .auth
        .refresh(cookie_header(&headers), request_context(&headers))
        .await?;
    let headers = cookie_headers(&state, &session.cookies)?;
    Ok((headers, Json(AuthUserResponse { user: session.user })))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    tag = "auth",
    operation_id = "logout",
    summary = "Log out",
    description = "Revokes the current access or refresh session when present and clears all auth cookies.",
    responses(
        (status = 204, description = "Session revoked and cookies cleared."),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<(HeaderMap, StatusCode)> {
    state.services.auth.logout(cookie_header(&headers)).await?;
    let headers = cookie_headers(&state, &state.services.auth.expired_cookies())?;
    Ok((headers, StatusCode::NO_CONTENT))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/password/forgot",
    tag = "auth",
    operation_id = "forgot_password",
    summary = "Request a password reset email",
    description = "Accepts a password reset request with enumeration-safe responses and sends a reset email when the account exists.",
    request_body(content = ForgotPasswordRequest),
    responses(
        (status = 202, description = "Password reset request accepted."),
        (status = 400, description = "Email is invalid.", body = crate::error::ErrorBody),
        (status = 502, description = "Email delivery failed.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn forgot_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ForgotPasswordRequest>,
) -> AppResult<StatusCode> {
    state
        .services
        .auth
        .forgot_password(request.into(), request_context(&headers))
        .await?;
    Ok(StatusCode::ACCEPTED)
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/password/reset",
    tag = "auth",
    operation_id = "reset_password",
    summary = "Reset password with a single-use token",
    description = "Consumes an unexpired single-use password reset token, updates the password hash, and revokes existing sessions.",
    request_body(content = ResetPasswordRequest),
    responses(
        (status = 204, description = "Password reset."),
        (status = 400, description = "Reset request is invalid.", body = crate::error::ErrorBody),
        (status = 401, description = "Reset token is invalid or expired.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn reset_password(
    State(state): State<AppState>,
    Json(request): Json<ResetPasswordRequest>,
) -> AppResult<StatusCode> {
    state.services.auth.reset_password(request.into()).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/sso/{provider}/start",
    tag = "auth",
    operation_id = "start_sso",
    summary = "Start an OAuth SSO flow",
    description = "Creates an OAuth state and PKCE verifier, stores them server-side, and redirects to the configured provider.",
    params(
        ("provider" = String, Path, description = "`github` or `gitlab`."),
        SsoStartQuery
    ),
    responses(
        (status = 303, description = "Redirect to provider authorization URL."),
        (status = 404, description = "Provider is not configured.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn start_sso(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Query(query): Query<SsoStartQuery>,
) -> AppResult<Redirect> {
    let start = state
        .services
        .auth
        .start_sso(&provider, query.redirect_to.as_deref())
        .await?;
    Ok(Redirect::to(&start.redirect_url))
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/sso/{provider}/callback",
    tag = "auth",
    operation_id = "finish_sso",
    summary = "Complete an OAuth SSO flow",
    description = "Validates OAuth state, exchanges the code with PKCE, resolves provider identity, starts a session, and redirects to the app.",
    params(
        ("provider" = String, Path, description = "`github` or `gitlab`."),
        SsoCallbackQuery
    ),
    responses(
        (status = 303, description = "Session cookies set and redirected to app."),
        (status = 401, description = "OAuth state is invalid or expired.", body = crate::error::ErrorBody),
        (status = 502, description = "OAuth provider request failed.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn finish_sso(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(provider): Path<String>,
    Query(query): Query<SsoCallbackQuery>,
) -> AppResult<Response> {
    let callback = state
        .services
        .auth
        .finish_sso(
            &provider,
            query.code.as_deref().unwrap_or_default(),
            query.state.as_deref().unwrap_or_default(),
            request_context(&headers),
        )
        .await?;
    let mut headers = HeaderMap::new();
    if let Some(cookies) = callback.cookies {
        headers = cookie_headers(&state, &cookies)?;
    }
    headers.insert(
        LOCATION,
        HeaderValue::from_str(&callback.redirect_to)
            .map_err(|_| AppError::BadRequest("invalid redirect"))?,
    );
    Ok((StatusCode::SEE_OTHER, headers).into_response())
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> AppResult<Response> {
    if is_public_request(request.method(), request.uri().path()) {
        return Ok(next.run(request).await);
    }

    let session = state
        .services
        .auth
        .current_session(cookie_header(request.headers()))
        .await?;

    if requires_csrf(request.method()) {
        state.services.auth.verify_csrf(
            &session,
            cookie_header(request.headers()),
            request
                .headers()
                .get("x-csrf-token")
                .and_then(|value| value.to_str().ok()),
        )?;
    }

    request.extensions_mut().insert(session.user);
    Ok(next.run(request).await)
}

fn is_public_request(method: &Method, path: &str) -> bool {
    if *method == Method::OPTIONS {
        return true;
    }
    if !path.starts_with("/api/v1/") {
        return true;
    }
    path == "/api/v1/meta"
        || path.starts_with("/api/v1/auth/")
        || path.starts_with("/api/v1/setup/")
}

fn requires_csrf(method: &Method) -> bool {
    !matches!(method, &Method::GET | &Method::HEAD | &Method::OPTIONS)
}

fn cookie_headers(state: &AppState, cookies: &AuthCookies) -> AppResult<HeaderMap> {
    let mut headers = HeaderMap::new();
    for cookie in [&cookies.access, &cookies.refresh, &cookies.csrf] {
        headers.append(
            SET_COOKIE,
            HeaderValue::from_str(&state.services.auth.cookie_header_value(cookie))
                .map_err(|_| AppError::BadRequest("invalid cookie"))?,
        );
    }
    Ok(headers)
}

fn cookie_header(headers: &HeaderMap) -> Option<&str> {
    headers.get(COOKIE).and_then(|value| value.to_str().ok())
}

fn request_context(headers: &HeaderMap) -> RequestContext {
    RequestContext {
        user_agent: headers
            .get("user-agent")
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned),
        ip_address: headers
            .get("x-forwarded-for")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split(',').next())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned),
    }
}

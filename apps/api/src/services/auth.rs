use std::time::Duration as StdDuration;

use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng as PasswordOsRng},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{Duration, Utc};
use rand::{Rng, RngCore, rngs::OsRng};
use sha2::{Digest, Sha256};
use url::Url;

use crate::{
    adapters::{
        oauth::{OAuthClient, OAuthTokenExchange},
        postgres::{
            PostgresAdapter,
            auth::{NewOAuthState, NewSession, NewSignupEmailVerification, OAuthIdentity},
        },
        resend::{EmailDeliveryClient, ResendSendEmail},
    },
    error::{AppError, AppResult},
    models::{AuthProvidersResponse, AuthUser, SsoProviderAvailability, SsoProviders},
};

const ACCESS_COOKIE: &str = "fs_access";
const REFRESH_COOKIE: &str = "fs_refresh";
const CSRF_COOKIE: &str = "fs_csrf";
const GITHUB_PROVIDER: &str = "github";
const GITLAB_PROVIDER: &str = "gitlab";
const DEFAULT_GITLAB_BASE_URL: &str = "https://gitlab.com";

#[derive(Debug, Clone)]
pub struct SignupStartInput {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct SignupCompleteInput {
    pub email: String,
    pub password: String,
    pub code: String,
}

#[derive(Debug, Clone)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct ForgotPasswordInput {
    pub email: String,
}

#[derive(Debug, Clone)]
pub struct ResetPasswordInput {
    pub token: String,
    pub password: String,
    pub password_confirmation: String,
}

#[derive(Debug, Clone, Default)]
pub struct RequestContext {
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedSession {
    pub session_id: i64,
    pub user: AuthUser,
    pub csrf_token_hash: String,
}

#[derive(Debug, Clone)]
pub struct LoginSession {
    pub user: AuthUser,
    pub cookies: AuthCookies,
}

#[derive(Debug, Clone)]
pub struct AuthCookies {
    pub access: CookieSpec,
    pub refresh: CookieSpec,
    pub csrf: CookieSpec,
}

#[derive(Debug, Clone)]
pub struct CookieSpec {
    pub name: &'static str,
    pub value: String,
    pub max_age: i64,
    pub http_only: bool,
}

#[derive(Debug, Clone)]
pub struct SsoStart {
    pub redirect_url: String,
}

#[derive(Debug, Clone)]
pub struct SsoCallback {
    pub redirect_to: String,
    pub cookies: AuthCookies,
}

#[derive(Clone)]
pub struct AuthService {
    postgres: PostgresAdapter,
    email_delivery: EmailDeliveryClient,
    oauth: OAuthClient,
    public_app_url: String,
    public_api_url: String,
    secrets_key: String,
    cookie_secure: bool,
    access_ttl: Duration,
    refresh_ttl: Duration,
    reset_ttl: Duration,
    signup_code_ttl: Duration,
    oauth_state_ttl: Duration,
}

impl AuthService {
    pub fn new(
        postgres: PostgresAdapter,
        email_delivery: EmailDeliveryClient,
        oauth: OAuthClient,
        public_app_url: String,
        public_api_url: String,
        secrets_key: String,
    ) -> Self {
        let public_app_url = public_app_url.trim_end_matches('/').to_owned();
        let public_api_url = public_api_url.trim_end_matches('/').to_owned();
        let cookie_secure = public_app_url.starts_with("https://");

        Self {
            postgres,
            email_delivery,
            oauth,
            public_app_url,
            public_api_url,
            secrets_key,
            cookie_secure,
            access_ttl: Duration::minutes(15),
            refresh_ttl: Duration::days(30),
            reset_ttl: Duration::hours(1),
            signup_code_ttl: Duration::minutes(10),
            oauth_state_ttl: Duration::minutes(10),
        }
    }

    pub async fn providers(&self) -> AppResult<AuthProvidersResponse> {
        let github = self.provider_availability(GITHUB_PROVIDER).await?;
        let gitlab = self.provider_availability(GITLAB_PROVIDER).await?;

        Ok(AuthProvidersResponse {
            password: true,
            sso: SsoProviders { github, gitlab },
        })
    }

    pub async fn start_signup(
        &self,
        input: SignupStartInput,
        context: RequestContext,
    ) -> AppResult<()> {
        let email = normalize_email(&input.email)?;
        validate_password(&input.password)?;

        if self.postgres.auth_user_by_email(&email).await?.is_some() {
            return Err(AppError::Conflict("account_exists"));
        }

        let config = self
            .postgres
            .get_reset_email_config(&self.secrets_key)
            .await?
            .ok_or(AppError::Conflict("email_delivery_not_configured"))?;

        let code = random_verification_code();
        self.postgres
            .create_signup_email_verification(NewSignupEmailVerification {
                email: &email,
                code_hash: &signup_code_hash(&email, &code),
                expires_at: Utc::now() + self.signup_code_ttl,
                user_agent: context.user_agent.as_deref(),
                ip_address: context.ip_address.as_deref(),
            })
            .await?;

        let html = format!(
            "<p>Your Fosslate verification code is:</p><p><strong>{code}</strong></p><p>This code expires in 10 minutes.</p>"
        );
        self.email_delivery
            .send_email(ResendSendEmail {
                api_key: &config.api_key,
                from_name: &config.from_name,
                from_email: &config.from_email,
                recipient: &email,
                subject: "Verify your Fosslate email",
                html: &html,
            })
            .await
            .map_err(|error| {
                tracing::warn!(%error, "signup verification email failed");
                AppError::ExternalService("email_delivery_failed")
            })?;

        self.postgres
            .record_auth_attempt(
                "signup_start",
                Some(&email),
                context.ip_address.as_deref(),
                true,
            )
            .await?;

        Ok(())
    }

    pub async fn complete_signup(
        &self,
        input: SignupCompleteInput,
        context: RequestContext,
    ) -> AppResult<LoginSession> {
        let email = normalize_email(&input.email)?;
        validate_password(&input.password)?;
        let code = normalize_verification_code(&input.code)?;

        if self.postgres.auth_user_by_email(&email).await?.is_some() {
            return Err(AppError::Conflict("account_exists"));
        }

        let verified = self
            .postgres
            .consume_signup_email_verification(&email, &signup_code_hash(&email, &code))
            .await?;
        if !verified {
            self.postgres
                .record_auth_attempt(
                    "signup_complete",
                    Some(&email),
                    context.ip_address.as_deref(),
                    false,
                )
                .await?;
            return Err(AppError::Unauthorized);
        }

        let password_hash = hash_password(&input.password)?;
        let username = username_from_email(&email);

        let user = self
            .postgres
            .create_auth_user(&email, &username, Some(&password_hash), false, None)
            .await?;

        self.postgres.update_last_login(user.id).await?;
        self.postgres
            .record_auth_attempt(
                "signup_complete",
                Some(&email),
                context.ip_address.as_deref(),
                true,
            )
            .await?;
        self.create_login_session(user, context).await
    }

    pub async fn login(
        &self,
        input: LoginInput,
        context: RequestContext,
    ) -> AppResult<LoginSession> {
        let email = normalize_email(&input.email)?;
        let user = self.postgres.auth_user_by_email(&email).await?;

        let Some(user) = user else {
            self.record_login_attempt(Some(&email), context.ip_address.as_deref(), false)
                .await;
            return Err(AppError::Unauthorized);
        };

        if user.disabled_at.is_some() {
            self.record_login_attempt(Some(&email), context.ip_address.as_deref(), false)
                .await;
            return Err(AppError::Unauthorized);
        }

        let Some(password_hash) = user.password_hash.as_deref() else {
            self.record_login_attempt(Some(&email), context.ip_address.as_deref(), false)
                .await;
            return Err(AppError::Unauthorized);
        };

        if !verify_password(&input.password, password_hash) {
            self.record_login_attempt(Some(&email), context.ip_address.as_deref(), false)
                .await;
            return Err(AppError::Unauthorized);
        }

        self.record_login_attempt(Some(&email), context.ip_address.as_deref(), true)
            .await;
        self.postgres.update_last_login(user.id).await?;
        let user = self.postgres.auth_user_by_id(user.id).await?;
        self.create_login_session(user, context).await
    }

    pub async fn current_session(
        &self,
        cookie_header: Option<&str>,
    ) -> AppResult<AuthenticatedSession> {
        let access_token =
            cookie_value(cookie_header, ACCESS_COOKIE).ok_or(AppError::Unauthorized)?;
        let token_hash = token_hash(&access_token);
        let Some(row) = self.postgres.session_by_access_token(&token_hash).await? else {
            return Err(AppError::Unauthorized);
        };

        self.postgres.touch_session(row.session_id).await?;

        Ok(AuthenticatedSession {
            session_id: row.session_id,
            csrf_token_hash: row.csrf_token_hash.clone(),
            user: row.into(),
        })
    }

    pub fn verify_csrf(
        &self,
        session: &AuthenticatedSession,
        cookie_header: Option<&str>,
        csrf_header: Option<&str>,
    ) -> AppResult<()> {
        let csrf_cookie = cookie_value(cookie_header, CSRF_COOKIE).ok_or(AppError::Forbidden)?;
        let csrf_header = csrf_header
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or(AppError::Forbidden)?;

        if csrf_cookie != csrf_header {
            return Err(AppError::Forbidden);
        }

        if token_hash(&csrf_cookie) != session.csrf_token_hash {
            return Err(AppError::Forbidden);
        }

        Ok(())
    }

    pub async fn refresh(
        &self,
        cookie_header: Option<&str>,
        context: RequestContext,
    ) -> AppResult<LoginSession> {
        let refresh_token =
            cookie_value(cookie_header, REFRESH_COOKIE).ok_or(AppError::Unauthorized)?;
        let token_hash = token_hash(&refresh_token);
        let Some(row) = self.postgres.session_by_refresh_token(&token_hash).await? else {
            return Err(AppError::Unauthorized);
        };

        self.postgres.revoke_session(row.session_id).await?;
        self.create_login_session(row.into(), context).await
    }

    pub async fn logout(&self, cookie_header: Option<&str>) -> AppResult<()> {
        if let Some(access_token) = cookie_value(cookie_header, ACCESS_COOKIE) {
            let token_hash = token_hash(&access_token);
            if let Some(row) = self.postgres.session_by_access_token(&token_hash).await? {
                self.postgres.revoke_session(row.session_id).await?;
                return Ok(());
            }
        }

        if let Some(refresh_token) = cookie_value(cookie_header, REFRESH_COOKIE) {
            let token_hash = token_hash(&refresh_token);
            if let Some(row) = self.postgres.session_by_refresh_token(&token_hash).await? {
                self.postgres.revoke_session(row.session_id).await?;
            }
        }

        Ok(())
    }

    pub async fn forgot_password(
        &self,
        input: ForgotPasswordInput,
        context: RequestContext,
    ) -> AppResult<()> {
        let email = normalize_email(&input.email)?;
        let user = self.postgres.auth_user_by_email(&email).await?;

        if let Some(user) = user.filter(|user| user.disabled_at.is_none()) {
            let reset_token = random_token();
            let reset_hash = token_hash(&reset_token);
            self.postgres
                .create_password_reset_token(
                    user.id,
                    &reset_hash,
                    Utc::now() + self.reset_ttl,
                    context.user_agent.as_deref(),
                    context.ip_address.as_deref(),
                )
                .await?;

            if let Some(config) = self
                .postgres
                .get_reset_email_config(&self.secrets_key)
                .await?
            {
                let reset_url = format!(
                    "{}/reset-password?token={}",
                    self.public_app_url,
                    urlencoding(&reset_token)
                );
                let html = format!(
                    "<p>Use this link to reset your Fosslate password:</p><p><a href=\"{reset_url}\">{reset_url}</a></p><p>This link expires in 1 hour.</p>"
                );
                self.email_delivery
                    .send_email(ResendSendEmail {
                        api_key: &config.api_key,
                        from_name: &config.from_name,
                        from_email: &config.from_email,
                        recipient: &user.email,
                        subject: "Reset your Fosslate password",
                        html: &html,
                    })
                    .await
                    .map_err(|error| {
                        tracing::warn!(%error, "password reset email failed");
                        AppError::ExternalService("email_delivery_failed")
                    })?;
            } else {
                tracing::warn!("password reset requested before email delivery is configured");
            }

            self.postgres
                .record_auth_attempt(
                    "password_reset",
                    Some(&email),
                    context.ip_address.as_deref(),
                    true,
                )
                .await?;
        } else {
            self.postgres
                .record_auth_attempt(
                    "password_reset",
                    Some(&email),
                    context.ip_address.as_deref(),
                    false,
                )
                .await?;
        }

        Ok(())
    }

    pub async fn reset_password(&self, input: ResetPasswordInput) -> AppResult<()> {
        validate_password_confirmation(&input.password, &input.password_confirmation)?;
        validate_password(&input.password)?;
        let token = input.token.trim();
        if token.is_empty() {
            return Err(AppError::BadRequest("reset token is required"));
        }

        let token_hash = token_hash(token);
        let Some(user_id) = self
            .postgres
            .consume_password_reset_token(&token_hash)
            .await?
        else {
            return Err(AppError::Unauthorized);
        };

        let password_hash = hash_password(&input.password)?;
        self.postgres
            .update_password_hash(user_id, &password_hash)
            .await?;
        self.postgres.revoke_sessions_for_user(user_id).await?;
        Ok(())
    }

    pub async fn start_sso(
        &self,
        provider: &str,
        redirect_to: Option<&str>,
    ) -> AppResult<SsoStart> {
        let provider = normalized_provider(provider)?;
        let config = self
            .postgres
            .get_enabled_provider_config(provider, &self.secrets_key)
            .await?
            .ok_or(AppError::NotFound("auth_provider"))?;

        let state = random_token();
        let code_verifier = random_token();
        let code_challenge = code_challenge(&code_verifier);
        let redirect_to = sanitize_redirect(redirect_to);
        let callback_url = self.callback_url(provider);

        self.postgres
            .create_oauth_state(NewOAuthState {
                state_hash: &token_hash(&state),
                provider,
                pkce_verifier: &code_verifier,
                redirect_to: &redirect_to,
                expires_at: Utc::now() + self.oauth_state_ttl,
                secrets_key: &self.secrets_key,
            })
            .await?;

        let scopes = config.scopes.join(" ");
        let redirect_url = if provider == GITHUB_PROVIDER {
            let mut url = Url::parse("https://github.com/login/oauth/authorize")
                .map_err(|_| AppError::BadRequest("invalid oauth url"))?;
            url.query_pairs_mut()
                .append_pair("client_id", &config.client_id)
                .append_pair("redirect_uri", &callback_url)
                .append_pair("scope", &scopes)
                .append_pair("state", &state)
                .append_pair("code_challenge", &code_challenge)
                .append_pair("code_challenge_method", "S256");
            url.to_string()
        } else {
            let base_url = config
                .base_url
                .as_deref()
                .unwrap_or(DEFAULT_GITLAB_BASE_URL)
                .trim_end_matches('/');
            let mut url = Url::parse(&format!("{base_url}/oauth/authorize"))
                .map_err(|_| AppError::BadRequest("invalid oauth url"))?;
            url.query_pairs_mut()
                .append_pair("client_id", &config.client_id)
                .append_pair("redirect_uri", &callback_url)
                .append_pair("response_type", "code")
                .append_pair("scope", &scopes)
                .append_pair("state", &state)
                .append_pair("code_challenge", &code_challenge)
                .append_pair("code_challenge_method", "S256");
            url.to_string()
        };

        Ok(SsoStart { redirect_url })
    }

    pub async fn finish_sso(
        &self,
        provider: &str,
        code: &str,
        state: &str,
        context: RequestContext,
    ) -> AppResult<SsoCallback> {
        let provider = normalized_provider(provider)?;
        let code = required(code, "oauth code is required")?;
        let state = required(state, "oauth state is required")?;
        let state_hash = token_hash(state);
        let state_row = self
            .postgres
            .consume_oauth_state(&state_hash, &self.secrets_key)
            .await?
            .ok_or(AppError::Unauthorized)?;

        if state_row.provider != provider {
            return Err(AppError::Unauthorized);
        }

        let config = self
            .postgres
            .get_enabled_provider_config(provider, &self.secrets_key)
            .await?
            .ok_or(AppError::NotFound("auth_provider"))?;
        let callback_url = self.callback_url(provider);

        let token = if provider == GITHUB_PROVIDER {
            self.oauth
                .exchange_github_code(OAuthTokenExchange {
                    client_id: &config.client_id,
                    client_secret: &config.client_secret,
                    code,
                    redirect_uri: &callback_url,
                    code_verifier: &state_row.pkce_verifier,
                })
                .await
        } else {
            let base_url = config
                .base_url
                .as_deref()
                .unwrap_or(DEFAULT_GITLAB_BASE_URL)
                .trim_end_matches('/')
                .to_owned();
            self.oauth
                .exchange_gitlab_code(
                    &base_url,
                    OAuthTokenExchange {
                        client_id: &config.client_id,
                        client_secret: &config.client_secret,
                        code,
                        redirect_uri: &callback_url,
                        code_verifier: &state_row.pkce_verifier,
                    },
                )
                .await
        }
        .map_err(|error| {
            tracing::warn!(%error, provider, "oauth token exchange failed");
            AppError::ExternalService("oauth_failed")
        })?;

        let identity = if provider == GITHUB_PROVIDER {
            self.oauth.github_identity(&token.access_token).await
        } else {
            let base_url = config
                .base_url
                .as_deref()
                .unwrap_or(DEFAULT_GITLAB_BASE_URL)
                .trim_end_matches('/')
                .to_owned();
            self.oauth
                .gitlab_identity(&base_url, &token.access_token)
                .await
        }
        .map_err(|error| {
            tracing::warn!(%error, provider, "oauth identity lookup failed");
            AppError::ExternalService("oauth_failed")
        })?;

        let user = self
            .postgres
            .upsert_oauth_user(OAuthIdentity {
                provider,
                provider_user_id: &identity.provider_user_id,
                email: &identity.email,
                username: &identity.username,
                avatar_url: identity.avatar_url.as_deref(),
            })
            .await?;

        let session = self.create_login_session(user, context).await?;

        Ok(SsoCallback {
            redirect_to: state_row.redirect_to,
            cookies: session.cookies,
        })
    }

    pub fn expired_cookies(&self) -> AuthCookies {
        AuthCookies {
            access: CookieSpec {
                name: ACCESS_COOKIE,
                value: String::new(),
                max_age: 0,
                http_only: true,
            },
            refresh: CookieSpec {
                name: REFRESH_COOKIE,
                value: String::new(),
                max_age: 0,
                http_only: true,
            },
            csrf: CookieSpec {
                name: CSRF_COOKIE,
                value: String::new(),
                max_age: 0,
                http_only: false,
            },
        }
    }

    pub fn cookie_header_value(&self, cookie: &CookieSpec) -> String {
        let mut value = format!(
            "{}={}; Path=/; Max-Age={}; SameSite=Lax",
            cookie.name, cookie.value, cookie.max_age
        );
        if cookie.http_only {
            value.push_str("; HttpOnly");
        }
        if self.cookie_secure {
            value.push_str("; Secure");
        }
        value
    }

    async fn provider_availability(&self, provider: &str) -> AppResult<SsoProviderAvailability> {
        let config = self.postgres.get_auth_provider_config(provider).await?;
        let enabled = config
            .as_ref()
            .map(|config| config.enabled && config.configured_at.is_some())
            .unwrap_or(false);
        let base_url = if provider == GITLAB_PROVIDER {
            config
                .and_then(|config| config.base_url)
                .or_else(|| Some(DEFAULT_GITLAB_BASE_URL.to_owned()))
        } else {
            None
        };

        Ok(SsoProviderAvailability {
            enabled,
            start_url: enabled.then(|| format!("/api/v1/auth/sso/{provider}/start")),
            base_url,
        })
    }

    async fn create_login_session(
        &self,
        user: AuthUser,
        context: RequestContext,
    ) -> AppResult<LoginSession> {
        let access_token = random_token();
        let refresh_token = random_token();
        let csrf_token = random_token();
        let now = Utc::now();
        let access_expires_at = now + self.access_ttl;
        let refresh_expires_at = now + self.refresh_ttl;

        self.postgres
            .create_auth_session(NewSession {
                user_id: user.id,
                access_token_hash: &token_hash(&access_token),
                refresh_token_hash: &token_hash(&refresh_token),
                csrf_token_hash: &token_hash(&csrf_token),
                access_expires_at,
                refresh_expires_at,
                user_agent: context.user_agent.as_deref(),
                ip_address: context.ip_address.as_deref(),
            })
            .await?;

        Ok(LoginSession {
            user,
            cookies: AuthCookies {
                access: CookieSpec {
                    name: ACCESS_COOKIE,
                    value: access_token,
                    max_age: duration_seconds(self.access_ttl),
                    http_only: true,
                },
                refresh: CookieSpec {
                    name: REFRESH_COOKIE,
                    value: refresh_token,
                    max_age: duration_seconds(self.refresh_ttl),
                    http_only: true,
                },
                csrf: CookieSpec {
                    name: CSRF_COOKIE,
                    value: csrf_token,
                    max_age: duration_seconds(self.refresh_ttl),
                    http_only: false,
                },
            },
        })
    }

    fn callback_url(&self, provider: &str) -> String {
        format!(
            "{}/api/v1/auth/sso/{provider}/callback",
            self.public_api_url
        )
    }

    async fn record_login_attempt(&self, email: Option<&str>, ip: Option<&str>, success: bool) {
        if let Err(error) = self
            .postgres
            .record_auth_attempt("login", email, ip, success)
            .await
        {
            tracing::warn!(%error, "failed to record login attempt");
        }
    }
}

fn normalize_email(email: &str) -> AppResult<String> {
    let email = email.trim().to_lowercase();
    if email.contains('@') && email.len() <= 320 {
        Ok(email)
    } else {
        Err(AppError::BadRequest("valid email is required"))
    }
}

fn validate_password(password: &str) -> AppResult<()> {
    if password.chars().count() < 8 {
        return Err(AppError::BadRequest("password is too short"));
    }
    if !password.chars().any(char::is_uppercase) {
        return Err(AppError::BadRequest("password requires uppercase"));
    }
    if !password
        .chars()
        .any(|ch| !ch.is_ascii_alphanumeric() && !ch.is_whitespace())
    {
        return Err(AppError::BadRequest("password requires special character"));
    }
    Ok(())
}

fn validate_password_confirmation(password: &str, confirmation: &str) -> AppResult<()> {
    if password != confirmation {
        Err(AppError::BadRequest("password confirmation does not match"))
    } else {
        Ok(())
    }
}

fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut PasswordOsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| AppError::BadRequest("password could not be hashed"))
}

fn verify_password(password: &str, password_hash: &str) -> bool {
    let Ok(parsed_hash) = PasswordHash::new(password_hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

fn random_token() -> String {
    let mut bytes = [0_u8; 32];
    OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn random_verification_code() -> String {
    format!("{:06}", OsRng.gen_range(0..1_000_000))
}

fn token_hash(token: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(token.as_bytes()))
}

fn signup_code_hash(email: &str, code: &str) -> String {
    token_hash(&format!("{email}:{code}"))
}

fn normalize_verification_code(code: &str) -> AppResult<String> {
    let code = code.trim();
    if code.len() == 6 && code.chars().all(|ch| ch.is_ascii_digit()) {
        Ok(code.to_owned())
    } else {
        Err(AppError::BadRequest("valid verification code is required"))
    }
}

fn code_challenge(code_verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(code_verifier.as_bytes()))
}

fn cookie_value(cookie_header: Option<&str>, name: &str) -> Option<String> {
    cookie_header?
        .split(';')
        .filter_map(|part| part.trim().split_once('='))
        .find_map(|(cookie_name, value)| (cookie_name == name).then(|| value.trim().to_owned()))
}

fn username_from_email(email: &str) -> String {
    let local_part = email.split('@').next().unwrap_or("user");
    let normalized = local_part
        .trim()
        .to_lowercase()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_owned();
    let base = if normalized.is_empty() {
        "user".to_owned()
    } else {
        normalized
    };
    format!("{base}-{}", &uuid::Uuid::new_v4().simple().to_string()[..8])
}

fn normalized_provider(provider: &str) -> AppResult<&'static str> {
    match provider {
        GITHUB_PROVIDER => Ok(GITHUB_PROVIDER),
        GITLAB_PROVIDER => Ok(GITLAB_PROVIDER),
        _ => Err(AppError::NotFound("auth_provider")),
    }
}

fn required<'a>(value: &'a str, message: &'static str) -> AppResult<&'a str> {
    let value = value.trim();
    if value.is_empty() {
        Err(AppError::BadRequest(message))
    } else {
        Ok(value)
    }
}

fn sanitize_redirect(value: Option<&str>) -> String {
    let value = value.unwrap_or("/").trim();
    if value.starts_with('/') && !value.starts_with("//") {
        value.to_owned()
    } else {
        "/".to_owned()
    }
}

fn duration_seconds(duration: Duration) -> i64 {
    duration
        .to_std()
        .unwrap_or_else(|_| StdDuration::from_secs(0))
        .as_secs() as i64
}

fn urlencoding(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

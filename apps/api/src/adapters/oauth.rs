use serde::Deserialize;

#[derive(Clone)]
pub struct OAuthClient {
    http: reqwest::Client,
}

impl OAuthClient {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
        }
    }

    pub async fn exchange_github_code(
        &self,
        request: OAuthTokenExchange<'_>,
    ) -> Result<OAuthTokenResponse, OAuthError> {
        let response = self
            .http
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .form(&[
                ("client_id", request.client_id),
                ("client_secret", request.client_secret),
                ("code", request.code),
                ("redirect_uri", request.redirect_uri),
                ("code_verifier", request.code_verifier),
            ])
            .send()
            .await?;

        parse_token_response(response).await
    }

    pub async fn exchange_gitlab_code(
        &self,
        base_url: &str,
        request: OAuthTokenExchange<'_>,
    ) -> Result<OAuthTokenResponse, OAuthError> {
        let token_url = format!("{}/oauth/token", base_url.trim_end_matches('/'));
        let response = self
            .http
            .post(token_url)
            .header("Accept", "application/json")
            .form(&[
                ("client_id", request.client_id),
                ("client_secret", request.client_secret),
                ("code", request.code),
                ("grant_type", "authorization_code"),
                ("redirect_uri", request.redirect_uri),
                ("code_verifier", request.code_verifier),
            ])
            .send()
            .await?;

        parse_token_response(response).await
    }

    pub async fn github_identity(&self, access_token: &str) -> Result<OAuthIdentity, OAuthError> {
        let user_response = self
            .http
            .get("https://api.github.com/user")
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "Fosslate")
            .bearer_auth(access_token)
            .send()
            .await?;
        let user: GithubUser = parse_json_response(user_response).await?;

        let emails_response = self
            .http
            .get("https://api.github.com/user/emails")
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "Fosslate")
            .bearer_auth(access_token)
            .send()
            .await?;
        let emails: Vec<GithubEmail> = parse_json_response(emails_response).await?;
        let email = emails
            .into_iter()
            .find(|email| email.primary && email.verified)
            .map(|email| email.email)
            .ok_or(OAuthError::MissingVerifiedEmail)?;

        Ok(OAuthIdentity {
            provider_user_id: user.id.to_string(),
            email,
            username: user.login,
            avatar_url: user.avatar_url,
        })
    }

    pub async fn gitlab_identity(
        &self,
        base_url: &str,
        access_token: &str,
    ) -> Result<OAuthIdentity, OAuthError> {
        let userinfo_url = format!("{}/oauth/userinfo", base_url.trim_end_matches('/'));
        let response = self
            .http
            .get(userinfo_url)
            .header("Accept", "application/json")
            .bearer_auth(access_token)
            .send()
            .await?;
        let user: GitlabUserInfo = parse_json_response(response).await?;

        if user.email.trim().is_empty() {
            return Err(OAuthError::MissingVerifiedEmail);
        }
        if user.email_verified == Some(false) {
            return Err(OAuthError::MissingVerifiedEmail);
        }

        Ok(OAuthIdentity {
            provider_user_id: user.sub,
            email: user.email,
            username: user
                .nickname
                .or(user.preferred_username)
                .or(user.name)
                .unwrap_or_else(|| "gitlab-user".to_owned()),
            avatar_url: user.picture,
        })
    }
}

impl Default for OAuthClient {
    fn default() -> Self {
        Self::new()
    }
}

pub struct OAuthTokenExchange<'a> {
    pub client_id: &'a str,
    pub client_secret: &'a str,
    pub code: &'a str,
    pub redirect_uri: &'a str,
    pub code_verifier: &'a str,
}

#[derive(Debug, Clone)]
pub struct OAuthTokenResponse {
    pub access_token: String,
}

#[derive(Debug, Clone)]
pub struct OAuthIdentity {
    pub provider_user_id: String,
    pub email: String,
    pub username: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ProviderTokenResponse {
    access_token: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GithubUser {
    id: i64,
    login: String,
    avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GithubEmail {
    email: String,
    primary: bool,
    verified: bool,
}

#[derive(Debug, Deserialize)]
struct GitlabUserInfo {
    sub: String,
    email: String,
    email_verified: Option<bool>,
    nickname: Option<String>,
    preferred_username: Option<String>,
    name: Option<String>,
    picture: Option<String>,
}

async fn parse_token_response(
    response: reqwest::Response,
) -> Result<OAuthTokenResponse, OAuthError> {
    let status = response.status();
    let body = response.text().await?;

    if !status.is_success() {
        return Err(OAuthError::Rejected(format!(
            "OAuth token endpoint returned HTTP {status}: {body}"
        )));
    }

    let body: ProviderTokenResponse =
        serde_json::from_str(&body).map_err(OAuthError::InvalidResponse)?;

    if let Some(error) = body.error {
        return Err(OAuthError::Rejected(
            body.error_description.unwrap_or(error),
        ));
    }

    let access_token = body
        .access_token
        .filter(|token| !token.trim().is_empty())
        .ok_or(OAuthError::MissingAccessToken)?;

    Ok(OAuthTokenResponse { access_token })
}

async fn parse_json_response<T: for<'de> Deserialize<'de>>(
    response: reqwest::Response,
) -> Result<T, OAuthError> {
    let status = response.status();
    let body = response.text().await?;

    if !status.is_success() {
        return Err(OAuthError::Rejected(format!(
            "OAuth provider returned HTTP {status}: {body}"
        )));
    }

    serde_json::from_str(&body).map_err(OAuthError::InvalidResponse)
}

#[derive(Debug, thiserror::Error)]
pub enum OAuthError {
    #[error("oauth request failed")]
    Request(#[from] reqwest::Error),
    #[error("oauth provider rejected the request: {0}")]
    Rejected(String),
    #[error("oauth provider returned invalid JSON")]
    InvalidResponse(serde_json::Error),
    #[error("oauth provider did not return an access token")]
    MissingAccessToken,
    #[error("oauth provider did not return a verified email address")]
    MissingVerifiedEmail,
}

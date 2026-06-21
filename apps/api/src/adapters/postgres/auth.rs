use chrono::{DateTime, Utc};

use crate::models::{AccountIdentity, AccountSsoProvider, AuthUser};

use super::PostgresAdapter;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PasswordUserRow {
    pub id: i64,
    pub email: String,
    pub username: String,
    pub password_hash: Option<String>,
    pub disabled_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SessionUserRow {
    pub session_id: i64,
    pub user_id: i64,
    pub csrf_token_hash: String,
    pub refresh_expires_at: DateTime<Utc>,
    pub id: i64,
    pub email: String,
    pub username: String,
    pub avatar_url: Option<String>,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AuthUserRow {
    pub id: i64,
    pub email: String,
    pub username: String,
    pub avatar_url: Option<String>,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EnabledProviderConfigRow {
    pub provider: String,
    pub base_url: Option<String>,
    pub client_id: String,
    pub client_secret: String,
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ResetEmailConfigRow {
    pub api_key: String,
    pub from_name: String,
    pub from_email: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct OAuthStateRow {
    pub provider: String,
    pub pkce_verifier: String,
    pub redirect_to: String,
    pub action: String,
    pub user_id: Option<i64>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AccountIdentityRow {
    pub provider: String,
    pub provider_user_id: String,
    pub user_id: i64,
    pub email: Option<String>,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewSession<'a> {
    pub user_id: i64,
    pub access_token_hash: &'a str,
    pub refresh_token_hash: &'a str,
    pub csrf_token_hash: &'a str,
    pub access_expires_at: DateTime<Utc>,
    pub refresh_expires_at: DateTime<Utc>,
    pub user_agent: Option<&'a str>,
    pub ip_address: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct NewOAuthState<'a> {
    pub state_hash: &'a str,
    pub provider: &'a str,
    pub pkce_verifier: &'a str,
    pub redirect_to: &'a str,
    pub action: &'a str,
    pub user_id: Option<i64>,
    pub expires_at: DateTime<Utc>,
    pub secrets_key: &'a str,
}

#[derive(Debug, Clone)]
pub struct NewSignupEmailVerification<'a> {
    pub email: &'a str,
    pub code_hash: &'a str,
    pub expires_at: DateTime<Utc>,
    pub user_agent: Option<&'a str>,
    pub ip_address: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct OAuthIdentity<'a> {
    pub provider: &'a str,
    pub provider_user_id: &'a str,
    pub email: &'a str,
    pub username: &'a str,
    pub avatar_url: Option<&'a str>,
}

impl From<SessionUserRow> for AuthUser {
    fn from(row: SessionUserRow) -> Self {
        Self {
            id: row.id,
            email: row.email,
            username: row.username,
            is_admin: row.is_admin,
            avatar_url: row.avatar_url,
            email_verified_at: row.email_verified_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<AuthUserRow> for AuthUser {
    fn from(row: AuthUserRow) -> Self {
        Self {
            id: row.id,
            email: row.email,
            username: row.username,
            is_admin: row.is_admin,
            avatar_url: row.avatar_url,
            email_verified_at: row.email_verified_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl TryFrom<AccountIdentityRow> for AccountIdentity {
    type Error = sqlx::Error;

    fn try_from(row: AccountIdentityRow) -> Result<Self, Self::Error> {
        let provider = match row.provider.as_str() {
            "github" => AccountSsoProvider::Github,
            "gitlab" => AccountSsoProvider::Gitlab,
            _ => {
                return Err(sqlx::Error::ColumnDecode {
                    index: "provider".to_owned(),
                    source: std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "unknown auth provider",
                    )
                    .into(),
                });
            }
        };

        Ok(Self {
            provider,
            email: row.email,
            username: row.username,
            avatar_url: row.avatar_url,
            connected_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl PostgresAdapter {
    pub async fn create_auth_user(
        &self,
        email: &str,
        username: &str,
        password_hash: Option<&str>,
        email_verified: bool,
        avatar_url: Option<&str>,
    ) -> Result<AuthUser, sqlx::Error> {
        let row = sqlx::query_as::<_, AuthUserRow>(
            r#"
            INSERT INTO users (
                email,
                username,
                password_hash,
                email_verified_at,
                avatar_url
            )
            VALUES ($1, $2, $3, CASE WHEN $4 THEN now() ELSE NULL END, $5)
            RETURNING
                id,
                email::text AS email,
                username,
                avatar_url,
                email_verified_at,
                is_admin,
                created_at,
                updated_at
            "#,
        )
        .bind(email)
        .bind(username)
        .bind(password_hash)
        .bind(email_verified)
        .bind(avatar_url)
        .fetch_one(self.pool())
        .await?;

        Ok(row.into())
    }

    pub async fn create_signup_email_verification(
        &self,
        input: NewSignupEmailVerification<'_>,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.pool().begin().await?;

        sqlx::query(
            r#"
            UPDATE signup_email_verifications
            SET used_at = now()
            WHERE email = $1
              AND used_at IS NULL
            "#,
        )
        .bind(input.email)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO signup_email_verifications (
                email,
                code_hash,
                expires_at,
                requested_user_agent,
                requested_ip_address
            )
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(input.email)
        .bind(input.code_hash)
        .bind(input.expires_at)
        .bind(input.user_agent)
        .bind(input.ip_address)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn consume_signup_email_verification(
        &self,
        email: &str,
        code_hash: &str,
    ) -> Result<bool, sqlx::Error> {
        let consumed_id = sqlx::query_scalar::<_, i64>(
            r#"
            UPDATE signup_email_verifications
            SET used_at = now()
            WHERE email = $1
              AND code_hash = $2
              AND used_at IS NULL
              AND expires_at > now()
            RETURNING id
            "#,
        )
        .bind(email)
        .bind(code_hash)
        .fetch_optional(self.pool())
        .await?;

        Ok(consumed_id.is_some())
    }

    pub async fn auth_user_by_email(
        &self,
        email: &str,
    ) -> Result<Option<PasswordUserRow>, sqlx::Error> {
        sqlx::query_as::<_, PasswordUserRow>(
            r#"
            SELECT
                id,
                email::text AS email,
                username,
                password_hash,
                disabled_at
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(self.pool())
        .await
    }

    pub async fn auth_user_by_id(&self, user_id: i64) -> Result<AuthUser, sqlx::Error> {
        let row = sqlx::query_as::<_, AuthUserRow>(
            r#"
            SELECT
                users.id,
                users.email::text AS email,
                users.username,
                users.avatar_url,
                users.email_verified_at,
                users.is_admin,
                users.created_at,
                users.updated_at
            FROM users
            WHERE users.id = $1
              AND users.email IS NOT NULL
              AND users.disabled_at IS NULL
            "#,
        )
        .bind(user_id)
        .fetch_one(self.pool())
        .await?;

        Ok(row.into())
    }

    pub async fn grant_user_admin(&self, user_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE users
            SET is_admin = true
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn update_last_login(&self, user_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE users
            SET last_login_at = now()
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn create_auth_session(&self, input: NewSession<'_>) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO auth_sessions (
                user_id,
                access_token_hash,
                refresh_token_hash,
                csrf_token_hash,
                access_expires_at,
                refresh_expires_at,
                user_agent,
                ip_address
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(input.user_id)
        .bind(input.access_token_hash)
        .bind(input.refresh_token_hash)
        .bind(input.csrf_token_hash)
        .bind(input.access_expires_at)
        .bind(input.refresh_expires_at)
        .bind(input.user_agent)
        .bind(input.ip_address)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn session_by_access_token(
        &self,
        access_token_hash: &str,
    ) -> Result<Option<SessionUserRow>, sqlx::Error> {
        sqlx::query_as::<_, SessionUserRow>(
            r#"
            SELECT
                auth_sessions.id AS session_id,
                auth_sessions.user_id,
                auth_sessions.csrf_token_hash,
                auth_sessions.refresh_expires_at,
                users.id,
                COALESCE(users.email::text, users.username || '@local.fosslate') AS email,
                users.username,
                users.avatar_url,
                users.email_verified_at,
                users.is_admin,
                users.created_at,
                users.updated_at
            FROM auth_sessions
            JOIN users ON users.id = auth_sessions.user_id
            WHERE auth_sessions.access_token_hash = $1
              AND auth_sessions.revoked_at IS NULL
              AND auth_sessions.access_expires_at > now()
              AND users.disabled_at IS NULL
            "#,
        )
        .bind(access_token_hash)
        .fetch_optional(self.pool())
        .await
    }

    pub async fn session_by_refresh_token(
        &self,
        refresh_token_hash: &str,
    ) -> Result<Option<SessionUserRow>, sqlx::Error> {
        sqlx::query_as::<_, SessionUserRow>(
            r#"
            SELECT
                auth_sessions.id AS session_id,
                auth_sessions.user_id,
                auth_sessions.csrf_token_hash,
                auth_sessions.refresh_expires_at,
                users.id,
                COALESCE(users.email::text, users.username || '@local.fosslate') AS email,
                users.username,
                users.avatar_url,
                users.email_verified_at,
                users.is_admin,
                users.created_at,
                users.updated_at
            FROM auth_sessions
            JOIN users ON users.id = auth_sessions.user_id
            WHERE auth_sessions.refresh_token_hash = $1
              AND auth_sessions.revoked_at IS NULL
              AND auth_sessions.refresh_expires_at > now()
              AND users.disabled_at IS NULL
            "#,
        )
        .bind(refresh_token_hash)
        .fetch_optional(self.pool())
        .await
    }

    pub async fn touch_session(&self, session_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE auth_sessions
            SET last_seen_at = now()
            WHERE id = $1
            "#,
        )
        .bind(session_id)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn revoke_session(&self, session_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE auth_sessions
            SET revoked_at = COALESCE(revoked_at, now())
            WHERE id = $1
            "#,
        )
        .bind(session_id)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn revoke_sessions_for_user(&self, user_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE auth_sessions
            SET revoked_at = COALESCE(revoked_at, now())
            WHERE user_id = $1
              AND revoked_at IS NULL
            "#,
        )
        .bind(user_id)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn update_password_hash(
        &self,
        user_id: i64,
        password_hash: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE users
            SET password_hash = $2
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .bind(password_hash)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn user_has_password(&self, user_id: i64) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar::<_, bool>(
            r#"
            SELECT password_hash IS NOT NULL
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(self.pool())
        .await
    }

    pub async fn account_identities(
        &self,
        user_id: i64,
    ) -> Result<Vec<AccountIdentity>, sqlx::Error> {
        let rows = sqlx::query_as::<_, AccountIdentityRow>(
            r#"
            SELECT
                provider,
                provider_user_id,
                user_id,
                email::text AS email,
                username,
                avatar_url,
                created_at,
                updated_at
            FROM auth_identities
            WHERE user_id = $1
            ORDER BY provider
            "#,
        )
        .bind(user_id)
        .fetch_all(self.pool())
        .await?;

        rows.into_iter().map(AccountIdentity::try_from).collect()
    }

    pub async fn account_identity_for_provider(
        &self,
        user_id: i64,
        provider: &str,
    ) -> Result<Option<AccountIdentityRow>, sqlx::Error> {
        sqlx::query_as::<_, AccountIdentityRow>(
            r#"
            SELECT
                provider,
                provider_user_id,
                user_id,
                email::text AS email,
                username,
                avatar_url,
                created_at,
                updated_at
            FROM auth_identities
            WHERE user_id = $1
              AND provider = $2
            "#,
        )
        .bind(user_id)
        .bind(provider)
        .fetch_optional(self.pool())
        .await
    }

    pub async fn oauth_identity_user_id(
        &self,
        provider: &str,
        provider_user_id: &str,
    ) -> Result<Option<i64>, sqlx::Error> {
        sqlx::query_scalar::<_, i64>(
            r#"
            SELECT user_id
            FROM auth_identities
            WHERE provider = $1
              AND provider_user_id = $2
            "#,
        )
        .bind(provider)
        .bind(provider_user_id)
        .fetch_optional(self.pool())
        .await
    }

    pub async fn upsert_account_identity(
        &self,
        user_id: i64,
        identity: OAuthIdentity<'_>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO auth_identities (
                provider,
                provider_user_id,
                user_id,
                email,
                username,
                avatar_url
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (provider, provider_user_id) DO UPDATE
            SET
                email = EXCLUDED.email,
                username = EXCLUDED.username,
                avatar_url = EXCLUDED.avatar_url
            "#,
        )
        .bind(identity.provider)
        .bind(identity.provider_user_id)
        .bind(user_id)
        .bind(identity.email)
        .bind(identity.username)
        .bind(identity.avatar_url)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn remove_account_identity(
        &self,
        user_id: i64,
        provider: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM auth_identities
            WHERE user_id = $1
              AND provider = $2
            "#,
        )
        .bind(user_id)
        .bind(provider)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn create_password_reset_token(
        &self,
        user_id: i64,
        token_hash: &str,
        expires_at: DateTime<Utc>,
        user_agent: Option<&str>,
        ip_address: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO password_reset_tokens (
                user_id,
                token_hash,
                expires_at,
                requested_user_agent,
                requested_ip_address
            )
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(user_id)
        .bind(token_hash)
        .bind(expires_at)
        .bind(user_agent)
        .bind(ip_address)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn consume_password_reset_token(
        &self,
        token_hash: &str,
    ) -> Result<Option<i64>, sqlx::Error> {
        sqlx::query_scalar::<_, i64>(
            r#"
            UPDATE password_reset_tokens
            SET used_at = now()
            WHERE token_hash = $1
              AND used_at IS NULL
              AND expires_at > now()
            RETURNING user_id
            "#,
        )
        .bind(token_hash)
        .fetch_optional(self.pool())
        .await
    }

    pub async fn record_auth_attempt(
        &self,
        kind: &str,
        email: Option<&str>,
        ip_address: Option<&str>,
        success: bool,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO auth_attempts (kind, email, ip_address, success)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(kind)
        .bind(email)
        .bind(ip_address)
        .bind(success)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn get_enabled_provider_config(
        &self,
        provider: &str,
        secrets_key: &str,
    ) -> Result<Option<EnabledProviderConfigRow>, sqlx::Error> {
        sqlx::query_as::<_, EnabledProviderConfigRow>(
            r#"
            SELECT
                provider,
                base_url,
                client_id,
                pgp_sym_decrypt(client_secret_ciphertext, $2) AS client_secret,
                scopes
            FROM auth_provider_configs
            WHERE provider = $1
              AND enabled = true
              AND configured_at IS NOT NULL
            "#,
        )
        .bind(provider)
        .bind(secrets_key)
        .fetch_optional(self.pool())
        .await
    }

    pub async fn get_reset_email_config(
        &self,
        secrets_key: &str,
    ) -> Result<Option<ResetEmailConfigRow>, sqlx::Error> {
        sqlx::query_as::<_, ResetEmailConfigRow>(
            r#"
            SELECT
                pgp_sym_decrypt(api_key_ciphertext, $1) AS api_key,
                from_name,
                from_email
            FROM email_delivery_config
            WHERE id = 1
            "#,
        )
        .bind(secrets_key)
        .fetch_optional(self.pool())
        .await
    }

    pub async fn create_oauth_state(&self, input: NewOAuthState<'_>) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO oauth_login_states (
                state_hash,
                provider,
                pkce_verifier_ciphertext,
                redirect_to,
                action,
                user_id,
                expires_at
            )
            VALUES ($1, $2, pgp_sym_encrypt($3, $8), $4, $5, $6, $7)
            "#,
        )
        .bind(input.state_hash)
        .bind(input.provider)
        .bind(input.pkce_verifier)
        .bind(input.redirect_to)
        .bind(input.action)
        .bind(input.user_id)
        .bind(input.expires_at)
        .bind(input.secrets_key)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn consume_oauth_state(
        &self,
        state_hash: &str,
        secrets_key: &str,
    ) -> Result<Option<OAuthStateRow>, sqlx::Error> {
        sqlx::query_as::<_, OAuthStateRow>(
            r#"
            UPDATE oauth_login_states
            SET used_at = now()
            WHERE state_hash = $1
              AND used_at IS NULL
              AND expires_at > now()
            RETURNING
                provider,
                pgp_sym_decrypt(pkce_verifier_ciphertext, $2) AS pkce_verifier,
                redirect_to,
                action,
                user_id
            "#,
        )
        .bind(state_hash)
        .bind(secrets_key)
        .fetch_optional(self.pool())
        .await
    }

    pub async fn upsert_oauth_user(
        &self,
        identity: OAuthIdentity<'_>,
    ) -> Result<AuthUser, sqlx::Error> {
        let mut tx = self.pool().begin().await?;

        let existing_user_id = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT user_id
            FROM auth_identities
            WHERE provider = $1
              AND provider_user_id = $2
            "#,
        )
        .bind(identity.provider)
        .bind(identity.provider_user_id)
        .fetch_optional(&mut *tx)
        .await?;

        let user_id = if let Some(user_id) = existing_user_id {
            user_id
        } else if let Some(user_id) = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT id
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(identity.email)
        .fetch_optional(&mut *tx)
        .await?
        {
            user_id
        } else {
            let username = available_username_in_tx(&mut tx, identity.username).await?;
            sqlx::query_scalar::<_, i64>(
                r#"
                INSERT INTO users (
                    email,
                    username,
                    email_verified_at,
                    avatar_url
                )
                VALUES ($1, $2, now(), $3)
                RETURNING id
                "#,
            )
            .bind(identity.email)
            .bind(username)
            .bind(identity.avatar_url)
            .fetch_one(&mut *tx)
            .await?
        };

        sqlx::query(
            r#"
            INSERT INTO auth_identities (
                provider,
                provider_user_id,
                user_id,
                email,
                username,
                avatar_url
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (provider, provider_user_id) DO UPDATE
            SET
                email = EXCLUDED.email,
                username = EXCLUDED.username,
                avatar_url = EXCLUDED.avatar_url
            "#,
        )
        .bind(identity.provider)
        .bind(identity.provider_user_id)
        .bind(user_id)
        .bind(identity.email)
        .bind(identity.username)
        .bind(identity.avatar_url)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            UPDATE users
            SET
                email_verified_at = COALESCE(email_verified_at, now()),
                avatar_url = COALESCE($2, avatar_url),
                last_login_at = now()
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .bind(identity.avatar_url)
        .execute(&mut *tx)
        .await?;

        let user = sqlx::query_as::<_, AuthUserRow>(
            r#"
            SELECT
                users.id,
                users.email::text AS email,
                users.username,
                users.avatar_url,
                users.email_verified_at,
                users.is_admin,
                users.created_at,
                users.updated_at
            FROM users
            WHERE users.id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(user.into())
    }
}

async fn available_username_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    preferred: &str,
) -> Result<String, sqlx::Error> {
    let base = normalize_username(preferred);
    for suffix in 0..1000 {
        let candidate = if suffix == 0 {
            base.clone()
        } else {
            format!("{base}-{suffix}")
        };
        let exists = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM users WHERE username = $1
            )
            "#,
        )
        .bind(&candidate)
        .fetch_one(&mut **tx)
        .await?;

        if !exists {
            return Ok(candidate);
        }
    }

    Ok(format!("{base}-{}", uuid::Uuid::new_v4().simple()))
}

fn normalize_username(value: &str) -> String {
    let mut normalized = value
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
        .collect::<String>();

    while normalized.contains("--") {
        normalized = normalized.replace("--", "-");
    }

    normalized = normalized.trim_matches('-').to_owned();

    if normalized.is_empty() {
        "user".to_owned()
    } else {
        normalized
    }
}

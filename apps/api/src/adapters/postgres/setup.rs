use chrono::{DateTime, Utc};

use super::PostgresAdapter;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AuthProviderConfigRow {
    pub provider: String,
    pub enabled: bool,
    pub skipped_at: Option<DateTime<Utc>>,
    pub base_url: Option<String>,
    pub client_id: Option<String>,
    pub client_secret_ciphertext: Option<Vec<u8>>,
    pub scopes: Vec<String>,
    pub configured_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EmailDeliveryConfigRow {
    pub provider: String,
    pub from_name: String,
    pub from_email: String,
    pub last_test_recipient: String,
    pub last_tested_at: DateTime<Utc>,
    pub last_test_message_id: String,
}

impl PostgresAdapter {
    pub async fn setup_completed(&self) -> Result<bool, sqlx::Error> {
        let completed_at = sqlx::query_scalar::<_, Option<DateTime<Utc>>>(
            r#"
            SELECT completed_at
            FROM instance_setup
            WHERE id = 1
            "#,
        )
        .fetch_one(self.pool())
        .await?;

        Ok(completed_at.is_some())
    }

    pub async fn complete_setup(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE instance_setup
            SET completed_at = COALESCE(completed_at, now())
            WHERE id = 1
            "#,
        )
        .execute(self.pool())
        .await?;

        Ok(())
    }

    pub async fn get_auth_provider_config(
        &self,
        provider: &str,
    ) -> Result<Option<AuthProviderConfigRow>, sqlx::Error> {
        sqlx::query_as::<_, AuthProviderConfigRow>(
            r#"
            SELECT
                provider,
                enabled,
                skipped_at,
                base_url,
                client_id,
                client_secret_ciphertext,
                scopes,
                configured_at
            FROM auth_provider_configs
            WHERE provider = $1
            "#,
        )
        .bind(provider)
        .fetch_optional(self.pool())
        .await
    }

    pub async fn configure_auth_provider(
        &self,
        provider: &str,
        base_url: Option<&str>,
        client_id: &str,
        client_secret: &str,
        scopes: &[String],
        secrets_key: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO auth_provider_configs (
                provider,
                enabled,
                skipped_at,
                base_url,
                client_id,
                client_secret_ciphertext,
                scopes,
                configured_at
            )
            VALUES ($1, true, NULL, $2, $3, pgp_sym_encrypt($4, $6), $5, now())
            ON CONFLICT (provider) DO UPDATE
            SET
                enabled = EXCLUDED.enabled,
                skipped_at = NULL,
                base_url = EXCLUDED.base_url,
                client_id = EXCLUDED.client_id,
                client_secret_ciphertext = EXCLUDED.client_secret_ciphertext,
                scopes = EXCLUDED.scopes,
                configured_at = now()
            "#,
        )
        .bind(provider)
        .bind(base_url)
        .bind(client_id)
        .bind(client_secret)
        .bind(scopes)
        .bind(secrets_key)
        .execute(self.pool())
        .await?;

        Ok(())
    }

    pub async fn configure_auth_provider_without_secret_change(
        &self,
        provider: &str,
        base_url: Option<&str>,
        client_id: &str,
        scopes: &[String],
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE auth_provider_configs
            SET
                enabled = true,
                skipped_at = NULL,
                base_url = $2,
                client_id = $3,
                scopes = $4,
                configured_at = now()
            WHERE provider = $1
              AND client_secret_ciphertext IS NOT NULL
            "#,
        )
        .bind(provider)
        .bind(base_url)
        .bind(client_id)
        .bind(scopes)
        .execute(self.pool())
        .await?;

        Ok(())
    }

    pub async fn skip_auth_provider(
        &self,
        provider: &str,
        scopes: &[String],
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO auth_provider_configs (
                provider,
                enabled,
                skipped_at,
                base_url,
                client_id,
                client_secret_ciphertext,
                scopes,
                configured_at
            )
            VALUES ($1, false, now(), NULL, NULL, NULL, $2, NULL)
            ON CONFLICT (provider) DO UPDATE
            SET
                enabled = false,
                skipped_at = now(),
                base_url = NULL,
                client_id = NULL,
                client_secret_ciphertext = NULL,
                scopes = EXCLUDED.scopes,
                configured_at = NULL
            "#,
        )
        .bind(provider)
        .bind(scopes)
        .execute(self.pool())
        .await?;

        Ok(())
    }

    pub async fn get_email_delivery_config(
        &self,
    ) -> Result<Option<EmailDeliveryConfigRow>, sqlx::Error> {
        sqlx::query_as::<_, EmailDeliveryConfigRow>(
            r#"
            SELECT
                provider,
                from_name,
                from_email,
                last_test_recipient,
                last_tested_at,
                last_test_message_id
            FROM email_delivery_config
            WHERE id = 1
            "#,
        )
        .fetch_optional(self.pool())
        .await
    }

    pub async fn configure_email_delivery(
        &self,
        api_key: &str,
        from_name: &str,
        from_email: &str,
        last_test_recipient: &str,
        last_test_message_id: &str,
        secrets_key: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO email_delivery_config (
                id,
                provider,
                api_key_ciphertext,
                from_name,
                from_email,
                last_test_recipient,
                last_tested_at,
                last_test_message_id,
                configured_at
            )
            VALUES (
                1,
                'resend',
                pgp_sym_encrypt($1, $6),
                $2,
                $3,
                $4,
                now(),
                $5,
                now()
            )
            ON CONFLICT (id) DO UPDATE
            SET
                provider = EXCLUDED.provider,
                api_key_ciphertext = EXCLUDED.api_key_ciphertext,
                from_name = EXCLUDED.from_name,
                from_email = EXCLUDED.from_email,
                last_test_recipient = EXCLUDED.last_test_recipient,
                last_tested_at = now(),
                last_test_message_id = EXCLUDED.last_test_message_id,
                configured_at = now()
            "#,
        )
        .bind(api_key)
        .bind(from_name)
        .bind(from_email)
        .bind(last_test_recipient)
        .bind(last_test_message_id)
        .bind(secrets_key)
        .execute(self.pool())
        .await?;

        Ok(())
    }
}

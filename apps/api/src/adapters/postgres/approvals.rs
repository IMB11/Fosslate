use sqlx::{Postgres, Transaction};

use crate::models::TranslationApproval;

use super::PostgresAdapter;

impl PostgresAdapter {
    pub async fn upsert_translation_approval_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        string_id: i64,
        target_language_id: i64,
        translation_id: i64,
        approved_by_user_id: i64,
    ) -> Result<TranslationApproval, sqlx::Error> {
        sqlx::query_as::<_, TranslationApproval>(
            r#"
            INSERT INTO translation_approvals (
                string_id,
                target_language_id,
                translation_id,
                approved_by_user_id
            )
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (string_id, target_language_id)
            DO UPDATE SET
                translation_id = EXCLUDED.translation_id,
                approved_by_user_id = EXCLUDED.approved_by_user_id,
                approved_at = now()
            RETURNING
                string_id,
                target_language_id,
                translation_id,
                approved_by_user_id,
                approved_at
            "#,
        )
        .bind(string_id)
        .bind(target_language_id)
        .bind(translation_id)
        .bind(approved_by_user_id)
        .fetch_one(&mut **tx)
        .await
    }

    pub async fn get_translation_approval_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        string_id: i64,
        target_language_id: i64,
    ) -> Result<Option<TranslationApproval>, sqlx::Error> {
        sqlx::query_as::<_, TranslationApproval>(
            r#"
            SELECT
                string_id,
                target_language_id,
                translation_id,
                approved_by_user_id,
                approved_at
            FROM translation_approvals
            WHERE string_id = $1
              AND target_language_id = $2
            "#,
        )
        .bind(string_id)
        .bind(target_language_id)
        .fetch_optional(&mut **tx)
        .await
    }

    pub async fn delete_translation_approval_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        string_id: i64,
        target_language_id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM translation_approvals
            WHERE string_id = $1
              AND target_language_id = $2
            "#,
        )
        .bind(string_id)
        .bind(target_language_id)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}

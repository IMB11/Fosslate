use sqlx::{Postgres, Transaction};

use crate::models::Translation;

use super::PostgresAdapter;

impl PostgresAdapter {
    pub async fn create_translation_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        string_id: i64,
        target_language_id: i64,
        author_user_id: i64,
        value: &str,
    ) -> Result<Translation, sqlx::Error> {
        sqlx::query_as::<_, Translation>(
            r#"
            INSERT INTO translations (
                project_id,
                namespace_id,
                string_id,
                target_language_id,
                author_user_id,
                value
            )
            SELECT
                source_strings.project_id,
                source_strings.namespace_id,
                source_strings.id,
                $2,
                $3,
                $4
            FROM source_strings
            JOIN project_target_languages
              ON project_target_languages.id = $2
             AND project_target_languages.project_id = source_strings.project_id
             AND project_target_languages.deleted_at IS NULL
            WHERE source_strings.id = $1
              AND source_strings.deleted_at IS NULL
            RETURNING
                id,
                project_id,
                namespace_id,
                string_id,
                target_language_id,
                author_user_id,
                value,
                rating_score,
                created_at
            "#,
        )
        .bind(string_id)
        .bind(target_language_id)
        .bind(author_user_id)
        .bind(value)
        .fetch_one(&mut **tx)
        .await
    }

    pub async fn list_translations(
        &self,
        string_id: i64,
        target_language_id: i64,
    ) -> Result<Vec<Translation>, sqlx::Error> {
        sqlx::query_as::<_, Translation>(
            r#"
            SELECT
                id,
                project_id,
                namespace_id,
                string_id,
                target_language_id,
                author_user_id,
                value,
                rating_score,
                created_at
            FROM translations
            WHERE string_id = $1
              AND target_language_id = $2
              AND deleted_at IS NULL
            ORDER BY rating_score DESC, created_at ASC, id ASC
            "#,
        )
        .bind(string_id)
        .bind(target_language_id)
        .fetch_all(self.pool())
        .await
    }

    pub async fn get_translation_for_update(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        translation_id: i64,
    ) -> Result<Translation, sqlx::Error> {
        sqlx::query_as::<_, Translation>(
            r#"
            SELECT
                id,
                project_id,
                namespace_id,
                string_id,
                target_language_id,
                author_user_id,
                value,
                rating_score,
                created_at
            FROM translations
            WHERE id = $1
              AND deleted_at IS NULL
            FOR UPDATE
            "#,
        )
        .bind(translation_id)
        .fetch_one(&mut **tx)
        .await
    }

    pub async fn increment_translation_rating_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        translation_id: i64,
        delta: i32,
    ) -> Result<Translation, sqlx::Error> {
        sqlx::query_as::<_, Translation>(
            r#"
            UPDATE translations
            SET rating_score = rating_score + $2
            WHERE id = $1
              AND deleted_at IS NULL
            RETURNING
                id,
                project_id,
                namespace_id,
                string_id,
                target_language_id,
                author_user_id,
                value,
                rating_score,
                created_at
            "#,
        )
        .bind(translation_id)
        .bind(delta)
        .fetch_one(&mut **tx)
        .await
    }

    pub async fn find_best_translation_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        string_id: i64,
        target_language_id: i64,
    ) -> Result<Option<Translation>, sqlx::Error> {
        sqlx::query_as::<_, Translation>(
            r#"
            SELECT
                id,
                project_id,
                namespace_id,
                string_id,
                target_language_id,
                author_user_id,
                value,
                rating_score,
                created_at
            FROM translations
            WHERE string_id = $1
              AND target_language_id = $2
              AND deleted_at IS NULL
            ORDER BY rating_score DESC, created_at ASC, id ASC
            LIMIT 1
            "#,
        )
        .bind(string_id)
        .bind(target_language_id)
        .fetch_optional(&mut **tx)
        .await
    }

    pub async fn soft_delete_translation_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        project_id: i64,
        translation_id: i64,
    ) -> Result<Translation, sqlx::Error> {
        sqlx::query_as::<_, Translation>(
            r#"
            UPDATE translations
            SET deleted_at = now()
            WHERE project_id = $1
              AND id = $2
              AND deleted_at IS NULL
            RETURNING
                id,
                project_id,
                namespace_id,
                string_id,
                target_language_id,
                author_user_id,
                value,
                rating_score,
                created_at
            "#,
        )
        .bind(project_id)
        .bind(translation_id)
        .fetch_one(&mut **tx)
        .await
    }
}

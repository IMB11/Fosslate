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
            JOIN projects
              ON projects.id = source_strings.project_id
             AND projects.deleted_at IS NULL
            JOIN namespaces
              ON namespaces.id = source_strings.namespace_id
             AND namespaces.project_id = source_strings.project_id
             AND namespaces.deleted_at IS NULL
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
                translations.id,
                translations.project_id,
                translations.namespace_id,
                translations.string_id,
                translations.target_language_id,
                translations.author_user_id,
                translations.value,
                translations.rating_score,
                translations.created_at
            FROM translations
            JOIN projects
              ON projects.id = translations.project_id
             AND projects.deleted_at IS NULL
            JOIN source_strings
              ON source_strings.id = translations.string_id
             AND source_strings.project_id = translations.project_id
             AND source_strings.namespace_id = translations.namespace_id
             AND source_strings.deleted_at IS NULL
            JOIN namespaces
              ON namespaces.id = translations.namespace_id
             AND namespaces.project_id = translations.project_id
             AND namespaces.deleted_at IS NULL
            JOIN project_target_languages
              ON project_target_languages.id = translations.target_language_id
             AND project_target_languages.project_id = translations.project_id
             AND project_target_languages.deleted_at IS NULL
            WHERE translations.string_id = $1
              AND translations.target_language_id = $2
              AND translations.deleted_at IS NULL
            ORDER BY translations.rating_score DESC, translations.created_at ASC, translations.id ASC
            "#,
        )
        .bind(string_id)
        .bind(target_language_id)
        .fetch_all(self.pool())
        .await
    }

    pub async fn get_translation(&self, translation_id: i64) -> Result<Translation, sqlx::Error> {
        sqlx::query_as::<_, Translation>(
            r#"
            SELECT
                translations.id,
                translations.project_id,
                translations.namespace_id,
                translations.string_id,
                translations.target_language_id,
                translations.author_user_id,
                translations.value,
                translations.rating_score,
                translations.created_at
            FROM translations
            JOIN projects
              ON projects.id = translations.project_id
             AND projects.deleted_at IS NULL
            JOIN source_strings
              ON source_strings.id = translations.string_id
             AND source_strings.project_id = translations.project_id
             AND source_strings.namespace_id = translations.namespace_id
             AND source_strings.deleted_at IS NULL
            JOIN namespaces
              ON namespaces.id = translations.namespace_id
             AND namespaces.project_id = translations.project_id
             AND namespaces.deleted_at IS NULL
            JOIN project_target_languages
              ON project_target_languages.id = translations.target_language_id
             AND project_target_languages.project_id = translations.project_id
             AND project_target_languages.deleted_at IS NULL
            WHERE translations.id = $1
              AND translations.deleted_at IS NULL
            "#,
        )
        .bind(translation_id)
        .fetch_one(self.pool())
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
                translations.id,
                translations.project_id,
                translations.namespace_id,
                translations.string_id,
                translations.target_language_id,
                translations.author_user_id,
                translations.value,
                translations.rating_score,
                translations.created_at
            FROM translations
            JOIN projects
              ON projects.id = translations.project_id
             AND projects.deleted_at IS NULL
            JOIN source_strings
              ON source_strings.id = translations.string_id
             AND source_strings.project_id = translations.project_id
             AND source_strings.namespace_id = translations.namespace_id
             AND source_strings.deleted_at IS NULL
            JOIN namespaces
              ON namespaces.id = translations.namespace_id
             AND namespaces.project_id = translations.project_id
             AND namespaces.deleted_at IS NULL
            JOIN project_target_languages
              ON project_target_languages.id = translations.target_language_id
             AND project_target_languages.project_id = translations.project_id
             AND project_target_languages.deleted_at IS NULL
            WHERE translations.id = $1
              AND translations.deleted_at IS NULL
            FOR UPDATE OF translations
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
            FROM projects, source_strings, namespaces, project_target_languages
            WHERE translations.id = $1
              AND translations.deleted_at IS NULL
              AND projects.id = translations.project_id
              AND projects.deleted_at IS NULL
              AND source_strings.id = translations.string_id
              AND source_strings.project_id = translations.project_id
              AND source_strings.namespace_id = translations.namespace_id
              AND source_strings.deleted_at IS NULL
              AND namespaces.id = translations.namespace_id
              AND namespaces.project_id = translations.project_id
              AND namespaces.deleted_at IS NULL
              AND project_target_languages.id = translations.target_language_id
              AND project_target_languages.project_id = translations.project_id
              AND project_target_languages.deleted_at IS NULL
            RETURNING
                translations.id,
                translations.project_id,
                translations.namespace_id,
                translations.string_id,
                translations.target_language_id,
                translations.author_user_id,
                translations.value,
                translations.rating_score,
                translations.created_at
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
                translations.id,
                translations.project_id,
                translations.namespace_id,
                translations.string_id,
                translations.target_language_id,
                translations.author_user_id,
                translations.value,
                translations.rating_score,
                translations.created_at
            FROM translations
            JOIN projects
              ON projects.id = translations.project_id
             AND projects.deleted_at IS NULL
            JOIN source_strings
              ON source_strings.id = translations.string_id
             AND source_strings.project_id = translations.project_id
             AND source_strings.namespace_id = translations.namespace_id
             AND source_strings.deleted_at IS NULL
            JOIN namespaces
              ON namespaces.id = translations.namespace_id
             AND namespaces.project_id = translations.project_id
             AND namespaces.deleted_at IS NULL
            JOIN project_target_languages
              ON project_target_languages.id = translations.target_language_id
             AND project_target_languages.project_id = translations.project_id
             AND project_target_languages.deleted_at IS NULL
            WHERE translations.string_id = $1
              AND translations.target_language_id = $2
              AND translations.deleted_at IS NULL
            ORDER BY translations.rating_score DESC, translations.created_at ASC, translations.id ASC
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
            FROM projects, source_strings, namespaces, project_target_languages
            WHERE translations.project_id = $1
              AND translations.id = $2
              AND translations.deleted_at IS NULL
              AND projects.id = translations.project_id
              AND projects.deleted_at IS NULL
              AND source_strings.id = translations.string_id
              AND source_strings.project_id = translations.project_id
              AND source_strings.namespace_id = translations.namespace_id
              AND source_strings.deleted_at IS NULL
              AND namespaces.id = translations.namespace_id
              AND namespaces.project_id = translations.project_id
              AND namespaces.deleted_at IS NULL
              AND project_target_languages.id = translations.target_language_id
              AND project_target_languages.project_id = translations.project_id
              AND project_target_languages.deleted_at IS NULL
            RETURNING
                translations.id,
                translations.project_id,
                translations.namespace_id,
                translations.string_id,
                translations.target_language_id,
                translations.author_user_id,
                translations.value,
                translations.rating_score,
                translations.created_at
            "#,
        )
        .bind(project_id)
        .bind(translation_id)
        .fetch_one(&mut **tx)
        .await
    }
}

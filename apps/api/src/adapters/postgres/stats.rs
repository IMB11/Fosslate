use sqlx::{Postgres, Transaction};

use crate::models::NamespaceLanguageStats;

use super::PostgresAdapter;

impl PostgresAdapter {
    #[allow(dead_code)]
    pub async fn rebuild_namespace_language_stats_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<u64, sqlx::Error> {
        sqlx::query("DELETE FROM namespace_language_stats")
            .execute(&mut **tx)
            .await?;

        let result = sqlx::query(
            r#"
            INSERT INTO namespace_language_stats (
                project_id,
                namespace_id,
                target_language_id,
                string_count,
                translated_count,
                approved_count,
                candidate_count,
                missing_count,
                updated_at
            )
            SELECT
                namespaces.project_id,
                namespaces.id,
                project_target_languages.id,
                count(DISTINCT source_strings.id)::integer,
                count(DISTINCT current_translations.string_id)::integer,
                count(DISTINCT translation_approvals.string_id)::integer,
                count(translations.id)::integer,
                (
                    count(DISTINCT source_strings.id)
                    - count(DISTINCT current_translations.string_id)
                )::integer,
                now()
            FROM namespaces
            JOIN project_target_languages
              ON project_target_languages.project_id = namespaces.project_id
             AND project_target_languages.deleted_at IS NULL
            LEFT JOIN source_strings
              ON source_strings.namespace_id = namespaces.id
             AND source_strings.deleted_at IS NULL
            LEFT JOIN current_translations
              ON current_translations.string_id = source_strings.id
             AND current_translations.target_language_id = project_target_languages.id
             AND current_translations.current_translation_id IS NOT NULL
            LEFT JOIN translation_approvals
              ON translation_approvals.string_id = source_strings.id
             AND translation_approvals.target_language_id = project_target_languages.id
            LEFT JOIN translations
              ON translations.string_id = source_strings.id
             AND translations.target_language_id = project_target_languages.id
             AND translations.deleted_at IS NULL
            WHERE namespaces.deleted_at IS NULL
            GROUP BY namespaces.project_id, namespaces.id, project_target_languages.id
            "#,
        )
        .execute(&mut **tx)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn refresh_all_namespace_language_stats_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        namespace_id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO namespace_language_stats (
                project_id,
                namespace_id,
                target_language_id,
                string_count,
                translated_count,
                approved_count,
                candidate_count,
                missing_count,
                updated_at
            )
            SELECT
                namespaces.project_id,
                namespaces.id,
                project_target_languages.id,
                count(DISTINCT source_strings.id)::integer,
                count(DISTINCT current_translations.string_id)::integer,
                count(DISTINCT translation_approvals.string_id)::integer,
                count(translations.id)::integer,
                (
                    count(DISTINCT source_strings.id)
                    - count(DISTINCT current_translations.string_id)
                )::integer,
                now()
            FROM namespaces
            JOIN project_target_languages
              ON project_target_languages.project_id = namespaces.project_id
             AND project_target_languages.deleted_at IS NULL
            LEFT JOIN source_strings
              ON source_strings.namespace_id = namespaces.id
             AND source_strings.deleted_at IS NULL
            LEFT JOIN current_translations
              ON current_translations.string_id = source_strings.id
             AND current_translations.target_language_id = project_target_languages.id
             AND current_translations.current_translation_id IS NOT NULL
            LEFT JOIN translation_approvals
              ON translation_approvals.string_id = source_strings.id
             AND translation_approvals.target_language_id = project_target_languages.id
            LEFT JOIN translations
              ON translations.string_id = source_strings.id
             AND translations.target_language_id = project_target_languages.id
             AND translations.deleted_at IS NULL
            WHERE namespaces.id = $1
              AND namespaces.deleted_at IS NULL
            GROUP BY namespaces.project_id, namespaces.id, project_target_languages.id
            ON CONFLICT (namespace_id, target_language_id)
            DO UPDATE SET
                string_count = EXCLUDED.string_count,
                translated_count = EXCLUDED.translated_count,
                approved_count = EXCLUDED.approved_count,
                candidate_count = EXCLUDED.candidate_count,
                missing_count = EXCLUDED.missing_count,
                updated_at = now()
            "#,
        )
        .bind(namespace_id)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn refresh_namespace_language_stats_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        namespace_id: i64,
        target_language_id: i64,
    ) -> Result<NamespaceLanguageStats, sqlx::Error> {
        sqlx::query_as::<_, NamespaceLanguageStats>(
            r#"
            INSERT INTO namespace_language_stats (
                project_id,
                namespace_id,
                target_language_id,
                string_count,
                translated_count,
                approved_count,
                candidate_count,
                missing_count,
                updated_at
            )
            SELECT
                namespaces.project_id,
                namespaces.id,
                project_target_languages.id,
                count(DISTINCT source_strings.id)::integer,
                count(DISTINCT current_translations.string_id)::integer,
                count(DISTINCT translation_approvals.string_id)::integer,
                count(translations.id)::integer,
                (
                    count(DISTINCT source_strings.id)
                    - count(DISTINCT current_translations.string_id)
                )::integer,
                now()
            FROM namespaces
            JOIN project_target_languages
              ON project_target_languages.project_id = namespaces.project_id
             AND project_target_languages.id = $2
             AND project_target_languages.deleted_at IS NULL
            LEFT JOIN source_strings
              ON source_strings.namespace_id = namespaces.id
             AND source_strings.deleted_at IS NULL
            LEFT JOIN current_translations
              ON current_translations.string_id = source_strings.id
             AND current_translations.target_language_id = project_target_languages.id
             AND current_translations.current_translation_id IS NOT NULL
            LEFT JOIN translation_approvals
              ON translation_approvals.string_id = source_strings.id
             AND translation_approvals.target_language_id = project_target_languages.id
            LEFT JOIN translations
              ON translations.string_id = source_strings.id
             AND translations.target_language_id = project_target_languages.id
             AND translations.deleted_at IS NULL
            WHERE namespaces.id = $1
              AND namespaces.deleted_at IS NULL
            GROUP BY namespaces.project_id, namespaces.id, project_target_languages.id
            ON CONFLICT (namespace_id, target_language_id)
            DO UPDATE SET
                string_count = EXCLUDED.string_count,
                translated_count = EXCLUDED.translated_count,
                approved_count = EXCLUDED.approved_count,
                candidate_count = EXCLUDED.candidate_count,
                missing_count = EXCLUDED.missing_count,
                updated_at = now()
            RETURNING
                project_id,
                namespace_id,
                target_language_id,
                string_count,
                translated_count,
                approved_count,
                candidate_count,
                missing_count,
                updated_at
            "#,
        )
        .bind(namespace_id)
        .bind(target_language_id)
        .fetch_one(&mut **tx)
        .await
    }

}

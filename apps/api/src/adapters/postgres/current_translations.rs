use sqlx::{Postgres, Transaction};

use crate::models::{CurrentTranslation, select_current_translation};

use super::PostgresAdapter;

impl PostgresAdapter {
    #[allow(dead_code)]
    pub async fn rebuild_current_translations_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<u64, sqlx::Error> {
        sqlx::query("DELETE FROM current_translations")
            .execute(&mut **tx)
            .await?;

        let result = sqlx::query(
            r#"
            WITH eligible_pairs AS (
                SELECT
                    source_strings.project_id,
                    source_strings.namespace_id,
                    source_strings.id AS string_id,
                    project_target_languages.id AS target_language_id
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
                  ON namespaces.id = source_strings.namespace_id
                 AND namespaces.project_id = source_strings.project_id
                 AND namespaces.deleted_at IS NULL
                JOIN project_target_languages
                  ON project_target_languages.id = translations.target_language_id
                 AND project_target_languages.project_id = translations.project_id
                 AND project_target_languages.deleted_at IS NULL
                GROUP BY
                    source_strings.project_id,
                    source_strings.namespace_id,
                    source_strings.id,
                    project_target_languages.id
            ),
            active_translations AS (
                SELECT
                    translations.id,
                    translations.project_id,
                    translations.namespace_id,
                    translations.string_id,
                    translations.target_language_id,
                    translations.rating_score,
                    translations.created_at
                FROM translations
                JOIN eligible_pairs
                  ON eligible_pairs.project_id = translations.project_id
                 AND eligible_pairs.namespace_id = translations.namespace_id
                 AND eligible_pairs.string_id = translations.string_id
                 AND eligible_pairs.target_language_id = translations.target_language_id
                WHERE translations.deleted_at IS NULL
            ),
            pairs AS (
                SELECT
                    eligible_pairs.project_id,
                    eligible_pairs.namespace_id,
                    eligible_pairs.string_id,
                    eligible_pairs.target_language_id,
                    count(active_translations.id)::integer AS candidate_count
                FROM eligible_pairs
                LEFT JOIN active_translations
                  ON active_translations.project_id = eligible_pairs.project_id
                 AND active_translations.namespace_id = eligible_pairs.namespace_id
                 AND active_translations.string_id = eligible_pairs.string_id
                 AND active_translations.target_language_id = eligible_pairs.target_language_id
                GROUP BY
                    eligible_pairs.project_id,
                    eligible_pairs.namespace_id,
                    eligible_pairs.string_id,
                    eligible_pairs.target_language_id
            ),
            best AS (
                SELECT DISTINCT ON (string_id, target_language_id)
                    id,
                    string_id,
                    target_language_id
                FROM active_translations
                ORDER BY
                    string_id,
                    target_language_id,
                    rating_score DESC,
                    created_at ASC,
                    id ASC
            ),
            approved AS (
                SELECT
                    translation_approvals.string_id,
                    translation_approvals.target_language_id,
                    active_translations.id
                FROM translation_approvals
                JOIN active_translations
                  ON active_translations.id = translation_approvals.translation_id
                 AND active_translations.string_id = translation_approvals.string_id
                 AND active_translations.target_language_id = translation_approvals.target_language_id
            )
            INSERT INTO current_translations (
                project_id,
                namespace_id,
                string_id,
                target_language_id,
                current_translation_id,
                approved_translation_id,
                best_rated_translation_id,
                candidate_count,
                updated_at
            )
            SELECT
                pairs.project_id,
                pairs.namespace_id,
                pairs.string_id,
                pairs.target_language_id,
                COALESCE(approved.id, best.id),
                approved.id,
                best.id,
                pairs.candidate_count,
                now()
            FROM pairs
            LEFT JOIN best
              ON best.string_id = pairs.string_id
             AND best.target_language_id = pairs.target_language_id
            LEFT JOIN approved
              ON approved.string_id = pairs.string_id
             AND approved.target_language_id = pairs.target_language_id
            "#,
        )
        .execute(&mut **tx)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn upsert_current_translation_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        project_id: i64,
        namespace_id: i64,
        string_id: i64,
        target_language_id: i64,
        approved_translation_id: Option<i64>,
        best_rated_translation_id: Option<i64>,
    ) -> Result<CurrentTranslation, sqlx::Error> {
        let current_translation_id =
            select_current_translation(approved_translation_id, best_rated_translation_id);

        sqlx::query_as::<_, CurrentTranslation>(
            r#"
            INSERT INTO current_translations (
                project_id,
                namespace_id,
                string_id,
                target_language_id,
                current_translation_id,
                approved_translation_id,
                best_rated_translation_id,
                candidate_count,
                updated_at
            )
            VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6,
                $7,
                (
                    SELECT count(*)::integer
                    FROM translations
                    WHERE string_id = $3
                      AND target_language_id = $4
                      AND deleted_at IS NULL
                ),
                now()
            )
            ON CONFLICT (string_id, target_language_id)
            DO UPDATE SET
                current_translation_id = EXCLUDED.current_translation_id,
                approved_translation_id = EXCLUDED.approved_translation_id,
                best_rated_translation_id = EXCLUDED.best_rated_translation_id,
                candidate_count = EXCLUDED.candidate_count,
                updated_at = now()
            RETURNING
                project_id,
                namespace_id,
                string_id,
                target_language_id,
                current_translation_id,
                approved_translation_id,
                best_rated_translation_id,
                candidate_count,
                updated_at
            "#,
        )
        .bind(project_id)
        .bind(namespace_id)
        .bind(string_id)
        .bind(target_language_id)
        .bind(current_translation_id)
        .bind(approved_translation_id)
        .bind(best_rated_translation_id)
        .fetch_one(&mut **tx)
        .await
    }
}

use sqlx::{Postgres, Transaction};

use crate::models::{Language, ProjectTargetLanguage};

use super::PostgresAdapter;

#[derive(sqlx::FromRow)]
struct ProjectTargetLanguageRow {
    id: i64,
    project_id: i64,
    language_key: String,
    language_name: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<ProjectTargetLanguageRow> for ProjectTargetLanguage {
    fn from(row: ProjectTargetLanguageRow) -> Self {
        Self {
            id: row.id,
            project_id: row.project_id,
            language: Language {
                key: row.language_key,
                name: row.language_name,
            },
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl PostgresAdapter {
    pub async fn add_target_language_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        project_id: i64,
        language: &Language,
    ) -> Result<ProjectTargetLanguage, sqlx::Error> {
        let row = sqlx::query_as::<_, ProjectTargetLanguageRow>(
            r#"
            INSERT INTO project_target_languages (project_id, language_key, language_name)
            VALUES ($1, $2, $3)
            RETURNING id, project_id, language_key, language_name, created_at, updated_at
            "#,
        )
        .bind(project_id)
        .bind(&language.key)
        .bind(&language.name)
        .fetch_one(&mut **tx)
        .await?;

        Ok(row.into())
    }

    pub async fn list_target_languages(
        &self,
        project_id: i64,
    ) -> Result<Vec<ProjectTargetLanguage>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ProjectTargetLanguageRow>(
            r#"
            SELECT id, project_id, language_key, language_name, created_at, updated_at
            FROM project_target_languages
            WHERE project_id = $1
              AND deleted_at IS NULL
            ORDER BY id
            "#,
        )
        .bind(project_id)
        .fetch_all(self.pool())
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn remove_target_language(
        &self,
        project_id: i64,
        target_language_id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE project_target_languages
            SET deleted_at = now()
            WHERE project_id = $1
              AND id = $2
              AND deleted_at IS NULL
            "#,
        )
        .bind(project_id)
        .bind(target_language_id)
        .execute(self.pool())
        .await?;

        Ok(())
    }
}

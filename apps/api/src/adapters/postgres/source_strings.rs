use crate::models::{KeysetPage, SourceString};
use sqlx::{Postgres, Transaction};

use super::PostgresAdapter;

impl PostgresAdapter {
    pub async fn create_source_string_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        project_id: i64,
        namespace_id: i64,
        identifier: &str,
        value: &str,
    ) -> Result<SourceString, sqlx::Error> {
        sqlx::query_as::<_, SourceString>(
            r#"
            INSERT INTO source_strings (project_id, namespace_id, identifier, value)
            VALUES ($1, $2, $3, $4)
            RETURNING id, project_id, namespace_id, identifier, value, created_at, updated_at
            "#,
        )
        .bind(project_id)
        .bind(namespace_id)
        .bind(identifier)
        .bind(value)
        .fetch_one(&mut **tx)
        .await
    }

    pub async fn list_source_strings(
        &self,
        namespace_id: i64,
        page: KeysetPage,
    ) -> Result<Vec<SourceString>, sqlx::Error> {
        sqlx::query_as::<_, SourceString>(
            r#"
            SELECT id, project_id, namespace_id, identifier, value, created_at, updated_at
            FROM source_strings
            WHERE namespace_id = $1
              AND deleted_at IS NULL
              AND ($2::bigint IS NULL OR id > $2)
            ORDER BY id
            LIMIT $3
            "#,
        )
        .bind(namespace_id)
        .bind(page.after_id)
        .bind(page.limit)
        .fetch_all(self.pool())
        .await
    }

    pub async fn get_source_string(
        &self,
        project_id: i64,
        string_id: i64,
    ) -> Result<SourceString, sqlx::Error> {
        sqlx::query_as::<_, SourceString>(
            r#"
            SELECT id, project_id, namespace_id, identifier, value, created_at, updated_at
            FROM source_strings
            WHERE project_id = $1
              AND id = $2
              AND deleted_at IS NULL
            "#,
        )
        .bind(project_id)
        .bind(string_id)
        .fetch_one(self.pool())
        .await
    }

    pub async fn update_source_string(
        &self,
        project_id: i64,
        string_id: i64,
        identifier: &str,
        value: &str,
    ) -> Result<SourceString, sqlx::Error> {
        sqlx::query_as::<_, SourceString>(
            r#"
            UPDATE source_strings
            SET identifier = $3, value = $4
            WHERE project_id = $1
              AND id = $2
              AND deleted_at IS NULL
            RETURNING id, project_id, namespace_id, identifier, value, created_at, updated_at
            "#,
        )
        .bind(project_id)
        .bind(string_id)
        .bind(identifier)
        .bind(value)
        .fetch_one(self.pool())
        .await
    }

    pub async fn soft_delete_source_string_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        project_id: i64,
        string_id: i64,
    ) -> Result<SourceString, sqlx::Error> {
        sqlx::query_as::<_, SourceString>(
            r#"
            UPDATE source_strings
            SET deleted_at = now()
            WHERE project_id = $1
              AND id = $2
              AND deleted_at IS NULL
            RETURNING id, project_id, namespace_id, identifier, value, created_at, updated_at
            "#,
        )
        .bind(project_id)
        .bind(string_id)
        .fetch_one(&mut **tx)
        .await
    }
}

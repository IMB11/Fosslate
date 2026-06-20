use crate::models::Namespace;
use sqlx::{Postgres, Transaction};

use super::PostgresAdapter;

impl PostgresAdapter {
    pub async fn create_namespace_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        project_id: i64,
        name: &str,
    ) -> Result<Namespace, sqlx::Error> {
        sqlx::query_as::<_, Namespace>(
            r#"
            INSERT INTO namespaces (project_id, name)
            VALUES ($1, $2)
            RETURNING id, project_id, name, created_at, updated_at
            "#,
        )
        .bind(project_id)
        .bind(name)
        .fetch_one(&mut **tx)
        .await
    }

    pub async fn list_namespaces(&self, project_id: i64) -> Result<Vec<Namespace>, sqlx::Error> {
        sqlx::query_as::<_, Namespace>(
            r#"
            SELECT id, project_id, name, created_at, updated_at
            FROM namespaces
            WHERE project_id = $1
              AND deleted_at IS NULL
            ORDER BY id
            "#,
        )
        .bind(project_id)
        .fetch_all(self.pool())
        .await
    }

    pub async fn get_namespace(
        &self,
        project_id: i64,
        namespace_id: i64,
    ) -> Result<Namespace, sqlx::Error> {
        sqlx::query_as::<_, Namespace>(
            r#"
            SELECT id, project_id, name, created_at, updated_at
            FROM namespaces
            WHERE project_id = $1
              AND id = $2
              AND deleted_at IS NULL
            "#,
        )
        .bind(project_id)
        .bind(namespace_id)
        .fetch_one(self.pool())
        .await
    }

    pub async fn update_namespace(
        &self,
        project_id: i64,
        namespace_id: i64,
        name: &str,
    ) -> Result<Namespace, sqlx::Error> {
        sqlx::query_as::<_, Namespace>(
            r#"
            UPDATE namespaces
            SET name = $3
            WHERE project_id = $1
              AND id = $2
              AND deleted_at IS NULL
            RETURNING id, project_id, name, created_at, updated_at
            "#,
        )
        .bind(project_id)
        .bind(namespace_id)
        .bind(name)
        .fetch_one(self.pool())
        .await
    }

    pub async fn soft_delete_namespace(
        &self,
        project_id: i64,
        namespace_id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE namespaces
            SET deleted_at = now()
            WHERE project_id = $1
              AND id = $2
              AND deleted_at IS NULL
            "#,
        )
        .bind(project_id)
        .bind(namespace_id)
        .execute(self.pool())
        .await?;

        Ok(())
    }
}

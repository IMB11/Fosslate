use uuid::Uuid;

use crate::models::{Language, Project};

use super::PostgresAdapter;

#[derive(sqlx::FromRow)]
pub struct ProjectRow {
    pub id: i64,
    pub public_id: Uuid,
    pub name: String,
    pub icon_asset_id: Option<i64>,
    pub source_language_key: String,
    pub source_language_name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<ProjectRow> for Project {
    fn from(row: ProjectRow) -> Self {
        Self {
            id: row.id,
            public_id: row.public_id,
            name: row.name,
            icon_asset_id: row.icon_asset_id,
            source_language: Language {
                key: row.source_language_key,
                name: row.source_language_name,
            },
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl PostgresAdapter {
    pub async fn create_project(
        &self,
        name: &str,
        icon_asset_id: Option<i64>,
        source_language: &Language,
    ) -> Result<Project, sqlx::Error> {
        let row = sqlx::query_as::<_, ProjectRow>(
            r#"
            INSERT INTO projects (
                name,
                icon_asset_id,
                source_language_key,
                source_language_name
            )
            VALUES ($1, $2, $3, $4)
            RETURNING
                id,
                public_id,
                name,
                icon_asset_id,
                source_language_key,
                source_language_name,
                created_at,
                updated_at
            "#,
        )
        .bind(name)
        .bind(icon_asset_id)
        .bind(&source_language.key)
        .bind(&source_language.name)
        .fetch_one(self.pool())
        .await?;

        Ok(row.into())
    }

    pub async fn list_projects(&self) -> Result<Vec<Project>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ProjectRow>(
            r#"
            SELECT
                id,
                public_id,
                name,
                icon_asset_id,
                source_language_key,
                source_language_name,
                created_at,
                updated_at
            FROM projects
            WHERE deleted_at IS NULL
            ORDER BY id
            "#,
        )
        .fetch_all(self.pool())
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    pub async fn get_project_by_public_id(&self, public_id: Uuid) -> Result<Project, sqlx::Error> {
        let row = sqlx::query_as::<_, ProjectRow>(
            r#"
            SELECT
                id,
                public_id,
                name,
                icon_asset_id,
                source_language_key,
                source_language_name,
                created_at,
                updated_at
            FROM projects
            WHERE public_id = $1
              AND deleted_at IS NULL
            "#,
        )
        .bind(public_id)
        .fetch_one(self.pool())
        .await?;

        Ok(row.into())
    }

    pub async fn update_project(
        &self,
        public_id: Uuid,
        name: &str,
        icon_asset_id: Option<i64>,
        source_language: &Language,
    ) -> Result<Project, sqlx::Error> {
        let row = sqlx::query_as::<_, ProjectRow>(
            r#"
            UPDATE projects
            SET
                name = $2,
                icon_asset_id = $3,
                source_language_key = $4,
                source_language_name = $5
            WHERE public_id = $1
              AND deleted_at IS NULL
            RETURNING
                id,
                public_id,
                name,
                icon_asset_id,
                source_language_key,
                source_language_name,
                created_at,
                updated_at
            "#,
        )
        .bind(public_id)
        .bind(name)
        .bind(icon_asset_id)
        .bind(&source_language.key)
        .bind(&source_language.name)
        .fetch_one(self.pool())
        .await?;

        Ok(row.into())
    }

    pub async fn soft_delete_project(&self, public_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE projects
            SET deleted_at = now()
            WHERE public_id = $1
              AND deleted_at IS NULL
            "#,
        )
        .bind(public_id)
        .execute(self.pool())
        .await?;

        Ok(())
    }
}

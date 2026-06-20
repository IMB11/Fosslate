use uuid::Uuid;

use crate::{
    adapters::postgres::PostgresAdapter,
    error::AppResult,
    models::{Language, Project},
};

#[derive(Debug, Clone)]
pub struct CreateProject {
    pub name: String,
    pub icon_asset_id: Option<i64>,
    pub source_language: Language,
}

#[derive(Debug, Clone)]
pub struct UpdateProject {
    pub name: String,
    pub icon_asset_id: Option<i64>,
    pub source_language: Language,
}

#[derive(Clone)]
pub struct ProjectService {
    postgres: PostgresAdapter,
}

impl ProjectService {
    pub fn new(postgres: PostgresAdapter) -> Self {
        Self { postgres }
    }

    pub async fn create_project(&self, input: CreateProject) -> AppResult<Project> {
        Ok(self
            .postgres
            .create_project(&input.name, input.icon_asset_id, &input.source_language)
            .await?)
    }

    pub async fn list_projects(&self) -> AppResult<Vec<Project>> {
        Ok(self.postgres.list_projects().await?)
    }

    pub async fn get_project(&self, public_id: Uuid) -> AppResult<Project> {
        Ok(self.postgres.get_project_by_public_id(public_id).await?)
    }

    pub async fn update_project(
        &self,
        public_id: Uuid,
        input: UpdateProject,
    ) -> AppResult<Project> {
        Ok(self
            .postgres
            .update_project(
                public_id,
                &input.name,
                input.icon_asset_id,
                &input.source_language,
            )
            .await?)
    }

    pub async fn delete_project(&self, public_id: Uuid) -> AppResult<()> {
        Ok(self.postgres.soft_delete_project(public_id).await?)
    }
}

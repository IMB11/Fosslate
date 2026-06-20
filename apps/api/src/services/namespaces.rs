use uuid::Uuid;

use crate::{adapters::postgres::PostgresAdapter, error::AppResult, models::Namespace};

#[derive(Clone)]
pub struct NamespaceService {
    postgres: PostgresAdapter,
}

impl NamespaceService {
    pub fn new(postgres: PostgresAdapter) -> Self {
        Self { postgres }
    }

    pub async fn create_namespace(
        &self,
        project_public_id: Uuid,
        name: String,
    ) -> AppResult<Namespace> {
        let project = self
            .postgres
            .get_project_by_public_id(project_public_id)
            .await?;
        Ok(self.postgres.create_namespace(project.id, &name).await?)
    }

    pub async fn list_namespaces(&self, project_public_id: Uuid) -> AppResult<Vec<Namespace>> {
        let project = self
            .postgres
            .get_project_by_public_id(project_public_id)
            .await?;
        Ok(self.postgres.list_namespaces(project.id).await?)
    }

    pub async fn get_namespace(
        &self,
        project_public_id: Uuid,
        namespace_id: i64,
    ) -> AppResult<Namespace> {
        let project = self
            .postgres
            .get_project_by_public_id(project_public_id)
            .await?;
        Ok(self
            .postgres
            .get_namespace(project.id, namespace_id)
            .await?)
    }

    pub async fn update_namespace(
        &self,
        project_public_id: Uuid,
        namespace_id: i64,
        name: String,
    ) -> AppResult<Namespace> {
        let project = self
            .postgres
            .get_project_by_public_id(project_public_id)
            .await?;
        Ok(self
            .postgres
            .update_namespace(project.id, namespace_id, &name)
            .await?)
    }

    pub async fn delete_namespace(
        &self,
        project_public_id: Uuid,
        namespace_id: i64,
    ) -> AppResult<()> {
        let project = self
            .postgres
            .get_project_by_public_id(project_public_id)
            .await?;
        Ok(self
            .postgres
            .soft_delete_namespace(project.id, namespace_id)
            .await?)
    }
}

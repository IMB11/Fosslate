use uuid::Uuid;

use crate::{
    adapters::postgres::PostgresAdapter,
    error::AppResult,
    models::{KeysetPage, SourceString},
};

#[derive(Clone)]
pub struct SourceStringService {
    postgres: PostgresAdapter,
}

impl SourceStringService {
    pub fn new(postgres: PostgresAdapter) -> Self {
        Self { postgres }
    }

    pub async fn create_source_string(
        &self,
        project_public_id: Uuid,
        namespace_id: i64,
        identifier: String,
        value: String,
    ) -> AppResult<SourceString> {
        let project = self
            .postgres
            .get_project_by_public_id(project_public_id)
            .await?;
        let mut tx = self.postgres.begin().await?;
        let source_string = self
            .postgres
            .create_source_string_in_tx(&mut tx, project.id, namespace_id, &identifier, &value)
            .await?;
        self.postgres
            .refresh_all_namespace_language_stats_in_tx(&mut tx, namespace_id)
            .await?;
        tx.commit().await?;
        Ok(source_string)
    }

    pub async fn list_source_strings(
        &self,
        project_public_id: Uuid,
        namespace_id: i64,
        page: KeysetPage,
    ) -> AppResult<Vec<SourceString>> {
        let project = self
            .postgres
            .get_project_by_public_id(project_public_id)
            .await?;
        self.postgres
            .get_namespace(project.id, namespace_id)
            .await?;
        Ok(self
            .postgres
            .list_source_strings(namespace_id, page)
            .await?)
    }

    pub async fn get_source_string(
        &self,
        project_public_id: Uuid,
        string_id: i64,
    ) -> AppResult<SourceString> {
        let project = self
            .postgres
            .get_project_by_public_id(project_public_id)
            .await?;
        Ok(self
            .postgres
            .get_source_string(project.id, string_id)
            .await?)
    }

    pub async fn update_source_string(
        &self,
        project_public_id: Uuid,
        string_id: i64,
        identifier: String,
        value: String,
    ) -> AppResult<SourceString> {
        let project = self
            .postgres
            .get_project_by_public_id(project_public_id)
            .await?;
        Ok(self
            .postgres
            .update_source_string(project.id, string_id, &identifier, &value)
            .await?)
    }

    pub async fn delete_source_string(
        &self,
        project_public_id: Uuid,
        string_id: i64,
    ) -> AppResult<()> {
        let project = self
            .postgres
            .get_project_by_public_id(project_public_id)
            .await?;
        let mut tx = self.postgres.begin().await?;
        let source_string = self
            .postgres
            .soft_delete_source_string_in_tx(&mut tx, project.id, string_id)
            .await?;
        self.postgres
            .refresh_all_namespace_language_stats_in_tx(&mut tx, source_string.namespace_id)
            .await?;
        tx.commit().await?;
        Ok(())
    }
}

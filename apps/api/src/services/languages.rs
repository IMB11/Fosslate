use uuid::Uuid;

use crate::{
    adapters::postgres::PostgresAdapter,
    error::AppResult,
    models::{Language, ProjectTargetLanguage},
};

#[derive(Clone)]
pub struct LanguageService {
    postgres: PostgresAdapter,
}

impl LanguageService {
    pub fn new(postgres: PostgresAdapter) -> Self {
        Self { postgres }
    }

    pub async fn add_target_language(
        &self,
        project_public_id: Uuid,
        language: Language,
    ) -> AppResult<ProjectTargetLanguage> {
        let project = self
            .postgres
            .get_project_by_public_id(project_public_id)
            .await?;
        Ok(self
            .postgres
            .add_target_language(project.id, &language)
            .await?)
    }

    pub async fn list_target_languages(
        &self,
        project_public_id: Uuid,
    ) -> AppResult<Vec<ProjectTargetLanguage>> {
        let project = self
            .postgres
            .get_project_by_public_id(project_public_id)
            .await?;
        Ok(self.postgres.list_target_languages(project.id).await?)
    }

    pub async fn remove_target_language(
        &self,
        project_public_id: Uuid,
        target_language_id: i64,
    ) -> AppResult<()> {
        let project = self
            .postgres
            .get_project_by_public_id(project_public_id)
            .await?;
        Ok(self
            .postgres
            .remove_target_language(project.id, target_language_id)
            .await?)
    }
}

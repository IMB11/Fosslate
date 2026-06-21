use uuid::Uuid;

use crate::{
    adapters::postgres::PostgresAdapter, error::AppResult, models::NamespaceLanguageStats,
};

#[derive(Clone)]
pub struct StatsService {
    postgres: PostgresAdapter,
}

impl StatsService {
    pub fn new(postgres: PostgresAdapter) -> Self {
        Self { postgres }
    }

    pub async fn list_namespace_language_stats(
        &self,
        project_public_id: Uuid,
    ) -> AppResult<Vec<NamespaceLanguageStats>> {
        let project = self
            .postgres
            .get_project_by_public_id(project_public_id)
            .await?;

        Ok(self
            .postgres
            .list_namespace_language_stats(project.id)
            .await?)
    }
}

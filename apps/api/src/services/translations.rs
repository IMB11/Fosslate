use uuid::Uuid;

use crate::{adapters::postgres::PostgresAdapter, error::AppResult, models::Translation};

#[derive(Clone)]
pub struct TranslationService {
    postgres: PostgresAdapter,
}

impl TranslationService {
    pub fn new(postgres: PostgresAdapter) -> Self {
        Self { postgres }
    }

    pub async fn create_translation(
        &self,
        project_public_id: Uuid,
        string_id: i64,
        target_language_id: i64,
        author_user_id: i64,
        value: String,
    ) -> AppResult<Translation> {
        let project = self
            .postgres
            .get_project_by_public_id(project_public_id)
            .await?;
        self.postgres
            .get_source_string(project.id, string_id)
            .await?;
        let mut tx = self.postgres.begin().await?;
        let translation = self
            .postgres
            .create_translation_in_tx(
                &mut tx,
                string_id,
                target_language_id,
                author_user_id,
                &value,
            )
            .await?;
        let best = self
            .postgres
            .find_best_translation_in_tx(
                &mut tx,
                translation.string_id,
                translation.target_language_id,
            )
            .await?;
        let approval = self
            .postgres
            .get_translation_approval_in_tx(
                &mut tx,
                translation.string_id,
                translation.target_language_id,
            )
            .await?;
        self.postgres
            .upsert_current_translation_in_tx(
                &mut tx,
                translation.project_id,
                translation.namespace_id,
                translation.string_id,
                translation.target_language_id,
                approval.map(|approval| approval.translation_id),
                best.map(|translation| translation.id),
            )
            .await?;
        self.postgres
            .refresh_namespace_language_stats_in_tx(
                &mut tx,
                translation.namespace_id,
                translation.target_language_id,
            )
            .await?;
        tx.commit().await?;

        Ok(translation)
    }

    pub async fn list_translations(
        &self,
        project_public_id: Uuid,
        string_id: i64,
        target_language_id: i64,
    ) -> AppResult<Vec<Translation>> {
        let project = self
            .postgres
            .get_project_by_public_id(project_public_id)
            .await?;
        self.postgres
            .get_source_string(project.id, string_id)
            .await?;
        Ok(self
            .postgres
            .list_translations(string_id, target_language_id)
            .await?)
    }

    pub async fn delete_translation(
        &self,
        project_public_id: Uuid,
        translation_id: i64,
    ) -> AppResult<()> {
        let project = self
            .postgres
            .get_project_by_public_id(project_public_id)
            .await?;
        let mut tx = self.postgres.begin().await?;
        let translation = self
            .postgres
            .soft_delete_translation_in_tx(&mut tx, project.id, translation_id)
            .await?;
        let approval = self
            .postgres
            .get_translation_approval_in_tx(
                &mut tx,
                translation.string_id,
                translation.target_language_id,
            )
            .await?;
        let approved_translation_id = if approval
            .as_ref()
            .is_some_and(|approval| approval.translation_id == translation.id)
        {
            self.postgres
                .delete_translation_approval_in_tx(
                    &mut tx,
                    translation.string_id,
                    translation.target_language_id,
                )
                .await?;
            None
        } else {
            approval.map(|approval| approval.translation_id)
        };
        let best = self
            .postgres
            .find_best_translation_in_tx(
                &mut tx,
                translation.string_id,
                translation.target_language_id,
            )
            .await?;
        self.postgres
            .upsert_current_translation_in_tx(
                &mut tx,
                translation.project_id,
                translation.namespace_id,
                translation.string_id,
                translation.target_language_id,
                approved_translation_id,
                best.map(|translation| translation.id),
            )
            .await?;
        self.postgres
            .refresh_namespace_language_stats_in_tx(
                &mut tx,
                translation.namespace_id,
                translation.target_language_id,
            )
            .await?;
        tx.commit().await?;
        Ok(())
    }
}

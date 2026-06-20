use crate::{
    adapters::postgres::PostgresAdapter,
    error::{AppError, AppResult},
    models::CurrentTranslation,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct ApprovalService {
    postgres: PostgresAdapter,
}

impl ApprovalService {
    pub fn new(postgres: PostgresAdapter) -> Self {
        Self { postgres }
    }

    pub async fn approve_translation(
        &self,
        project_public_id: Uuid,
        translation_id: i64,
        approved_by_user_id: i64,
    ) -> AppResult<CurrentTranslation> {
        let project = self
            .postgres
            .get_project_by_public_id(project_public_id)
            .await?;
        let mut tx = self.postgres.begin().await?;
        let translation = self
            .postgres
            .get_translation_for_update(&mut tx, translation_id)
            .await?;
        if translation.project_id != project.id {
            return Err(AppError::NotFound("translation"));
        }
        self.postgres
            .upsert_translation_approval_in_tx(
                &mut tx,
                translation.string_id,
                translation.target_language_id,
                translation.id,
                approved_by_user_id,
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
        let current = self
            .postgres
            .upsert_current_translation_in_tx(
                &mut tx,
                translation.project_id,
                translation.namespace_id,
                translation.string_id,
                translation.target_language_id,
                Some(translation.id),
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

        Ok(current)
    }

    pub async fn remove_approval(
        &self,
        project_public_id: Uuid,
        string_id: i64,
        target_language_id: i64,
    ) -> AppResult<CurrentTranslation> {
        let project = self
            .postgres
            .get_project_by_public_id(project_public_id)
            .await?;
        self.postgres
            .get_source_string(project.id, string_id)
            .await?;
        let mut tx = self.postgres.begin().await?;
        let approval = self
            .postgres
            .get_translation_approval_in_tx(&mut tx, string_id, target_language_id)
            .await?
            .ok_or(AppError::NotFound("approval"))?;
        let approved_translation = self
            .postgres
            .get_translation_for_update(&mut tx, approval.translation_id)
            .await?;
        let best = self
            .postgres
            .find_best_translation_in_tx(&mut tx, string_id, target_language_id)
            .await?;
        self.postgres
            .delete_translation_approval_in_tx(&mut tx, string_id, target_language_id)
            .await?;
        let current = self
            .postgres
            .upsert_current_translation_in_tx(
                &mut tx,
                approved_translation.project_id,
                approved_translation.namespace_id,
                approved_translation.string_id,
                approved_translation.target_language_id,
                None,
                best.map(|translation| translation.id),
            )
            .await?;
        self.postgres
            .refresh_namespace_language_stats_in_tx(
                &mut tx,
                approved_translation.namespace_id,
                approved_translation.target_language_id,
            )
            .await?;
        tx.commit().await?;

        Ok(current)
    }
}

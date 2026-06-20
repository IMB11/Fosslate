use crate::{
    adapters::postgres::PostgresAdapter,
    error::{AppError, AppResult},
    models::{Translation, VoteValue, vote_delta},
};
use uuid::Uuid;

#[derive(Clone)]
pub struct VoteService {
    postgres: PostgresAdapter,
}

impl VoteService {
    pub fn new(postgres: PostgresAdapter) -> Self {
        Self { postgres }
    }

    pub async fn set_vote(
        &self,
        project_public_id: Uuid,
        translation_id: i64,
        user_id: i64,
        vote: i16,
    ) -> AppResult<Translation> {
        let vote = VoteValue::new(vote).ok_or(AppError::BadRequest("vote must be -1 or 1"))?;
        let project = self
            .postgres
            .get_project_by_public_id(project_public_id)
            .await?;
        let mut tx = self.postgres.begin().await?;
        let locked_translation = self
            .postgres
            .get_translation_for_update(&mut tx, translation_id)
            .await?;
        if locked_translation.project_id != project.id {
            return Err(AppError::NotFound("translation"));
        }
        let previous = self
            .postgres
            .get_translation_vote_for_update(&mut tx, translation_id, user_id)
            .await?
            .and_then(VoteValue::new);
        self.postgres
            .upsert_translation_vote_in_tx(&mut tx, translation_id, user_id, vote.value())
            .await?;
        let delta = vote_delta(vote, previous);
        let translation = self
            .postgres
            .increment_translation_rating_in_tx(&mut tx, translation_id, delta)
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
        tx.commit().await?;

        Ok(translation)
    }
}

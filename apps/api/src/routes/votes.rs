use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{app::AppState, error::AppResult, models::Translation};

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SetVoteRequest {
    pub user_id: i64,
    pub vote: i16,
}

#[utoipa::path(
    put,
    path = "/api/v1/projects/{project_public_id}/translations/{translation_id}/vote",
    tag = "votes",
    params(
        ("project_public_id" = Uuid, Path, description = "Project public ID"),
        ("translation_id" = i64, Path, description = "Translation ID")
    ),
    request_body = SetVoteRequest,
    responses((status = 200, description = "Vote saved", body = Translation))
)]
pub async fn set_vote(
    State(state): State<AppState>,
    Path((project_public_id, translation_id)): Path<(Uuid, i64)>,
    Json(request): Json<SetVoteRequest>,
) -> AppResult<Json<Translation>> {
    Ok(Json(
        state
            .services
            .votes
            .set_vote(
                project_public_id,
                translation_id,
                request.user_id,
                request.vote,
            )
            .await?,
    ))
}

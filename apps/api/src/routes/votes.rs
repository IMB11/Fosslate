use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    app::AppState,
    error::{AppError, AppResult},
    models::Translation,
    routes::auth::CurrentUser,
};

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SetVoteRequest {
    /// User ID casting or replacing the vote.
    pub user_id: i64,
    /// Vote value. Use `1` for an upvote and `-1` for a downvote.
    pub vote: i16,
}

#[utoipa::path(
    put,
    path = "/api/v1/projects/{project_public_id}/translations/{translation_id}/vote",
    tag = "votes",
    operation_id = "set_vote",
    summary = "Set a translation vote",
    description = "Creates or replaces the calling user's vote for a translation candidate, updates the candidate rating score by the vote delta, and recalculates the best/current translation projection.",
    params(
        ("project_public_id" = Uuid, Path, description = "Public UUID of the project that owns the translation candidate."),
        ("translation_id" = i64, Path, description = "Numeric translation candidate ID to vote on.")
    ),
    request_body(content = SetVoteRequest, description = "Vote payload. `vote` must be either `1` or `-1`."),
    responses(
        (status = 200, description = "Vote saved and translation rating updated.", body = Translation),
        (status = 400, description = "`vote` was not `1` or `-1`.", body = crate::error::ErrorBody),
        (status = 404, description = "Project, translation candidate, or user was not found, or a parent resource has been deleted.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn set_vote(
    State(state): State<AppState>,
    CurrentUser(current_user): CurrentUser,
    Path((project_public_id, translation_id)): Path<(Uuid, i64)>,
    Json(request): Json<SetVoteRequest>,
) -> AppResult<Json<Translation>> {
    if current_user.id != request.user_id {
        return Err(AppError::Forbidden);
    }

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

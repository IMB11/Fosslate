use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{app::AppState, error::AppResult, models::CurrentTranslation};

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ApproveTranslationRequest {
    /// Reviewer user ID that approved this translation candidate.
    pub approved_by_user_id: i64,
}

#[utoipa::path(
    put,
    path = "/api/v1/projects/{project_public_id}/translations/{translation_id}/approval",
    tag = "approvals",
    operation_id = "approve_translation",
    summary = "Approve a translation candidate",
    description = "Marks a translation candidate as the approved translation for its source string and target language, replacing any previous approval and recalculating current translation stats.",
    params(
        ("project_public_id" = Uuid, Path, description = "Public UUID of the project that owns the translation candidate."),
        ("translation_id" = i64, Path, description = "Numeric translation candidate ID to approve.")
    ),
    request_body(content = ApproveTranslationRequest, description = "Reviewer user ID to record on the approval."),
    responses(
        (status = 200, description = "Translation approved and current translation projection updated.", body = CurrentTranslation),
        (status = 404, description = "Project, translation candidate, or reviewer user was not found, or a parent resource has been deleted.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn approve_translation(
    State(state): State<AppState>,
    Path((project_public_id, translation_id)): Path<(Uuid, i64)>,
    Json(request): Json<ApproveTranslationRequest>,
) -> AppResult<Json<CurrentTranslation>> {
    Ok(Json(
        state
            .services
            .approvals
            .approve_translation(
                project_public_id,
                translation_id,
                request.approved_by_user_id,
            )
            .await?,
    ))
}

#[utoipa::path(
    delete,
    path = "/api/v1/projects/{project_public_id}/strings/{string_id}/approvals/{target_language_id}",
    tag = "approvals",
    operation_id = "remove_approval",
    summary = "Remove a translation approval",
    description = "Removes the approved translation for a source string and target language, then falls back the current translation to the best-rated active candidate when one exists.",
    params(
        ("project_public_id" = Uuid, Path, description = "Public UUID of the project that owns the source string."),
        ("string_id" = i64, Path, description = "Numeric source string ID whose approval should be removed."),
        ("target_language_id" = i64, Path, description = "Numeric target language ID whose approval should be removed.")
    ),
    responses(
        (status = 200, description = "Approval removed and current translation projection updated.", body = CurrentTranslation),
        (status = 404, description = "Project, source string, target language, or approval was not found, or a parent resource has been deleted.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn remove_approval(
    State(state): State<AppState>,
    Path((project_public_id, string_id, target_language_id)): Path<(Uuid, i64, i64)>,
) -> AppResult<Json<CurrentTranslation>> {
    Ok(Json(
        state
            .services
            .approvals
            .remove_approval(project_public_id, string_id, target_language_id)
            .await?,
    ))
}

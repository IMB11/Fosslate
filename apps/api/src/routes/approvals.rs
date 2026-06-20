use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{app::AppState, error::AppResult, models::CurrentTranslation};

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ApproveTranslationRequest {
    pub approved_by_user_id: i64,
}

#[utoipa::path(
    put,
    path = "/api/v1/projects/{project_public_id}/translations/{translation_id}/approval",
    tag = "approvals",
    params(
        ("project_public_id" = Uuid, Path, description = "Project public ID"),
        ("translation_id" = i64, Path, description = "Translation ID")
    ),
    request_body = ApproveTranslationRequest,
    responses((status = 200, description = "Translation approved", body = CurrentTranslation))
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
    params(
        ("project_public_id" = Uuid, Path, description = "Project public ID"),
        ("string_id" = i64, Path, description = "Source string ID"),
        ("target_language_id" = i64, Path, description = "Target language ID")
    ),
    responses((status = 200, description = "Approval removed", body = CurrentTranslation))
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

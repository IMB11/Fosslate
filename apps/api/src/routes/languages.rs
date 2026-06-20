use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    app::AppState,
    error::AppResult,
    models::{Language, ProjectTargetLanguage},
};

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AddTargetLanguageRequest {
    pub language: Language,
}

#[utoipa::path(
    post,
    path = "/api/v1/projects/{project_public_id}/languages",
    tag = "languages",
    params(("project_public_id" = Uuid, Path, description = "Project public ID")),
    request_body = AddTargetLanguageRequest,
    responses((status = 201, description = "Target language added", body = ProjectTargetLanguage))
)]
pub async fn add_target_language(
    State(state): State<AppState>,
    Path(project_public_id): Path<Uuid>,
    Json(request): Json<AddTargetLanguageRequest>,
) -> AppResult<(StatusCode, Json<ProjectTargetLanguage>)> {
    let language = state
        .services
        .languages
        .add_target_language(project_public_id, request.language)
        .await?;
    Ok((StatusCode::CREATED, Json(language)))
}

#[utoipa::path(
    get,
    path = "/api/v1/projects/{project_public_id}/languages",
    tag = "languages",
    params(("project_public_id" = Uuid, Path, description = "Project public ID")),
    responses((status = 200, description = "Target languages", body = [ProjectTargetLanguage]))
)]
pub async fn list_target_languages(
    State(state): State<AppState>,
    Path(project_public_id): Path<Uuid>,
) -> AppResult<Json<Vec<ProjectTargetLanguage>>> {
    Ok(Json(
        state
            .services
            .languages
            .list_target_languages(project_public_id)
            .await?,
    ))
}

#[utoipa::path(
    delete,
    path = "/api/v1/projects/{project_public_id}/languages/{target_language_id}",
    tag = "languages",
    params(
        ("project_public_id" = Uuid, Path, description = "Project public ID"),
        ("target_language_id" = i64, Path, description = "Target language ID")
    ),
    responses((status = 204, description = "Target language removed"))
)]
pub async fn remove_target_language(
    State(state): State<AppState>,
    Path((project_public_id, target_language_id)): Path<(Uuid, i64)>,
) -> AppResult<StatusCode> {
    state
        .services
        .languages
        .remove_target_language(project_public_id, target_language_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

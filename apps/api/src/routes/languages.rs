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
    /// Target language to add to the project.
    pub language: Language,
}

#[utoipa::path(
    post,
    path = "/api/v1/projects/{project_public_id}/languages",
    tag = "languages",
    operation_id = "add_target_language",
    summary = "Add a target language",
    description = "Adds a target language to an active project and initializes namespace-language stats for existing namespaces.",
    params(("project_public_id" = Uuid, Path, description = "Public UUID of the project that will receive the target language.")),
    request_body(content = AddTargetLanguageRequest, description = "Language key and display name to add."),
    responses(
        (status = 201, description = "Target language added.", body = ProjectTargetLanguage),
        (status = 404, description = "Project was not found or has been deleted.", body = crate::error::ErrorBody),
        (status = 409, description = "An active target language with the same key already exists in this project.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
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
    operation_id = "list_target_languages",
    summary = "List target languages",
    description = "Returns active target languages for one project ordered by target language ID.",
    params(("project_public_id" = Uuid, Path, description = "Public UUID of the project whose target languages should be listed.")),
    responses(
        (status = 200, description = "Active target languages ordered by ID.", body = [ProjectTargetLanguage]),
        (status = 404, description = "Project was not found or has been deleted.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
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
    operation_id = "remove_target_language",
    summary = "Remove a target language",
    description = "Soft-deletes a target language from the project. Removing an unknown or already-deleted target language is treated as a no-op once the project exists.",
    params(
        ("project_public_id" = Uuid, Path, description = "Public UUID of the project that owns the target language."),
        ("target_language_id" = i64, Path, description = "Numeric target language ID returned by the language list or create endpoint.")
    ),
    responses(
        (status = 204, description = "Target language removal accepted."),
        (status = 404, description = "Project was not found or has been deleted.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
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

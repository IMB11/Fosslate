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
    models::{Language, Project},
    services::projects::{CreateProject, UpdateProject},
};

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateProjectRequest {
    /// Human-readable project name shown in project lists and detail views.
    pub name: String,
    /// Optional asset ID for the project icon. `null` means no icon is attached.
    pub icon_asset_id: Option<i64>,
    /// Source language that original strings are authored in.
    pub source_language: Language,
}

impl From<CreateProjectRequest> for CreateProject {
    fn from(request: CreateProjectRequest) -> Self {
        Self {
            name: request.name,
            icon_asset_id: request.icon_asset_id,
            source_language: request.source_language,
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateProjectRequest {
    /// Replacement human-readable project name.
    pub name: String,
    /// Replacement icon asset ID. Send `null` to clear the icon.
    pub icon_asset_id: Option<i64>,
    /// Replacement source language for newly authored source strings.
    pub source_language: Language,
}

impl From<UpdateProjectRequest> for UpdateProject {
    fn from(request: UpdateProjectRequest) -> Self {
        Self {
            name: request.name,
            icon_asset_id: request.icon_asset_id,
            source_language: request.source_language,
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/projects",
    tag = "projects",
    operation_id = "create_project",
    summary = "Create a project",
    description = "Creates a translation project and returns both the internal numeric ID and the public UUID used by client-facing project routes.",
    request_body(content = CreateProjectRequest, description = "Project attributes to create."),
    responses(
        (status = 201, description = "Project created.", body = Project),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn create_project(
    State(state): State<AppState>,
    Json(request): Json<CreateProjectRequest>,
) -> AppResult<(StatusCode, Json<Project>)> {
    let project = state
        .services
        .projects
        .create_project(request.into())
        .await?;
    Ok((StatusCode::CREATED, Json(project)))
}

#[utoipa::path(
    get,
    path = "/api/v1/projects",
    tag = "projects",
    operation_id = "list_projects",
    summary = "List projects",
    description = "Returns all active projects ordered by internal project ID. Soft-deleted projects are omitted.",
    responses(
        (status = 200, description = "Active projects ordered by ID.", body = [Project]),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn list_projects(State(state): State<AppState>) -> AppResult<Json<Vec<Project>>> {
    Ok(Json(state.services.projects.list_projects().await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/projects/{project_public_id}",
    tag = "projects",
    operation_id = "get_project",
    summary = "Get a project",
    description = "Fetches one active project by its public UUID.",
    params(("project_public_id" = Uuid, Path, description = "Public project UUID returned as `public_id` when the project is created.")),
    responses(
        (status = 200, description = "Project found.", body = Project),
        (status = 404, description = "Project was not found or has been deleted.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn get_project(
    State(state): State<AppState>,
    Path(project_public_id): Path<Uuid>,
) -> AppResult<Json<Project>> {
    Ok(Json(
        state
            .services
            .projects
            .get_project(project_public_id)
            .await?,
    ))
}

#[utoipa::path(
    put,
    path = "/api/v1/projects/{project_public_id}",
    tag = "projects",
    operation_id = "update_project",
    summary = "Update a project",
    description = "Replaces the editable project fields for an active project.",
    params(("project_public_id" = Uuid, Path, description = "Public project UUID returned as `public_id` when the project is created.")),
    request_body(content = UpdateProjectRequest, description = "Replacement project attributes."),
    responses(
        (status = 200, description = "Project updated.", body = Project),
        (status = 404, description = "Project was not found or has been deleted.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn update_project(
    State(state): State<AppState>,
    Path(project_public_id): Path<Uuid>,
    Json(request): Json<UpdateProjectRequest>,
) -> AppResult<Json<Project>> {
    Ok(Json(
        state
            .services
            .projects
            .update_project(project_public_id, request.into())
            .await?,
    ))
}

#[utoipa::path(
    delete,
    path = "/api/v1/projects/{project_public_id}",
    tag = "projects",
    operation_id = "delete_project",
    summary = "Delete a project",
    description = "Soft-deletes the project so it no longer appears in project reads. The operation is idempotent for already-deleted or unknown project UUIDs.",
    params(("project_public_id" = Uuid, Path, description = "Public project UUID returned as `public_id` when the project is created.")),
    responses(
        (status = 204, description = "Project delete accepted."),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn delete_project(
    State(state): State<AppState>,
    Path(project_public_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    state
        .services
        .projects
        .delete_project(project_public_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

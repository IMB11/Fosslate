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
    pub name: String,
    pub icon_asset_id: Option<i64>,
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
    pub name: String,
    pub icon_asset_id: Option<i64>,
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
    request_body = CreateProjectRequest,
    responses((status = 201, description = "Project created", body = Project))
)]
pub async fn create_project(
    State(state): State<AppState>,
    Json(request): Json<CreateProjectRequest>,
) -> AppResult<(StatusCode, Json<Project>)> {
    let project = state.services.projects.create_project(request.into()).await?;
    Ok((StatusCode::CREATED, Json(project)))
}

#[utoipa::path(
    get,
    path = "/api/v1/projects",
    tag = "projects",
    responses((status = 200, description = "Projects", body = [Project]))
)]
pub async fn list_projects(State(state): State<AppState>) -> AppResult<Json<Vec<Project>>> {
    Ok(Json(state.services.projects.list_projects().await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/projects/{project_public_id}",
    tag = "projects",
    params(("project_public_id" = Uuid, Path, description = "Project public ID")),
    responses((status = 200, description = "Project", body = Project))
)]
pub async fn get_project(
    State(state): State<AppState>,
    Path(project_public_id): Path<Uuid>,
) -> AppResult<Json<Project>> {
    Ok(Json(state.services.projects.get_project(project_public_id).await?))
}

#[utoipa::path(
    put,
    path = "/api/v1/projects/{project_public_id}",
    tag = "projects",
    params(("project_public_id" = Uuid, Path, description = "Project public ID")),
    request_body = UpdateProjectRequest,
    responses((status = 200, description = "Project updated", body = Project))
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
    params(("project_public_id" = Uuid, Path, description = "Project public ID")),
    responses((status = 204, description = "Project deleted"))
)]
pub async fn delete_project(
    State(state): State<AppState>,
    Path(project_public_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    state.services.projects.delete_project(project_public_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

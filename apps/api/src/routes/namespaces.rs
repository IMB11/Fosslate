use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{app::AppState, error::AppResult, models::Namespace};

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct NamespaceRequest {
    pub name: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/projects/{project_public_id}/namespaces",
    tag = "namespaces",
    params(("project_public_id" = Uuid, Path, description = "Project public ID")),
    request_body = NamespaceRequest,
    responses((status = 201, description = "Namespace created", body = Namespace))
)]
pub async fn create_namespace(
    State(state): State<AppState>,
    Path(project_public_id): Path<Uuid>,
    Json(request): Json<NamespaceRequest>,
) -> AppResult<(StatusCode, Json<Namespace>)> {
    let namespace = state
        .services
        .namespaces
        .create_namespace(project_public_id, request.name)
        .await?;
    Ok((StatusCode::CREATED, Json(namespace)))
}

#[utoipa::path(
    get,
    path = "/api/v1/projects/{project_public_id}/namespaces",
    tag = "namespaces",
    params(("project_public_id" = Uuid, Path, description = "Project public ID")),
    responses((status = 200, description = "Namespaces", body = [Namespace]))
)]
pub async fn list_namespaces(
    State(state): State<AppState>,
    Path(project_public_id): Path<Uuid>,
) -> AppResult<Json<Vec<Namespace>>> {
    Ok(Json(
        state
            .services
            .namespaces
            .list_namespaces(project_public_id)
            .await?,
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/projects/{project_public_id}/namespaces/{namespace_id}",
    tag = "namespaces",
    params(
        ("project_public_id" = Uuid, Path, description = "Project public ID"),
        ("namespace_id" = i64, Path, description = "Namespace ID")
    ),
    responses((status = 200, description = "Namespace", body = Namespace))
)]
pub async fn get_namespace(
    State(state): State<AppState>,
    Path((project_public_id, namespace_id)): Path<(Uuid, i64)>,
) -> AppResult<Json<Namespace>> {
    Ok(Json(
        state
            .services
            .namespaces
            .get_namespace(project_public_id, namespace_id)
            .await?,
    ))
}

#[utoipa::path(
    put,
    path = "/api/v1/projects/{project_public_id}/namespaces/{namespace_id}",
    tag = "namespaces",
    params(
        ("project_public_id" = Uuid, Path, description = "Project public ID"),
        ("namespace_id" = i64, Path, description = "Namespace ID")
    ),
    request_body = NamespaceRequest,
    responses((status = 200, description = "Namespace updated", body = Namespace))
)]
pub async fn update_namespace(
    State(state): State<AppState>,
    Path((project_public_id, namespace_id)): Path<(Uuid, i64)>,
    Json(request): Json<NamespaceRequest>,
) -> AppResult<Json<Namespace>> {
    Ok(Json(
        state
            .services
            .namespaces
            .update_namespace(project_public_id, namespace_id, request.name)
            .await?,
    ))
}

#[utoipa::path(
    delete,
    path = "/api/v1/projects/{project_public_id}/namespaces/{namespace_id}",
    tag = "namespaces",
    params(
        ("project_public_id" = Uuid, Path, description = "Project public ID"),
        ("namespace_id" = i64, Path, description = "Namespace ID")
    ),
    responses((status = 204, description = "Namespace deleted"))
)]
pub async fn delete_namespace(
    State(state): State<AppState>,
    Path((project_public_id, namespace_id)): Path<(Uuid, i64)>,
) -> AppResult<StatusCode> {
    state
        .services
        .namespaces
        .delete_namespace(project_public_id, namespace_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

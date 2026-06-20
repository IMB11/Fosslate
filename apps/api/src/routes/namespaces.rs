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
    /// Namespace name unique within the active project, for example `common` or `checkout`.
    pub name: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/projects/{project_public_id}/namespaces",
    tag = "namespaces",
    operation_id = "create_namespace",
    summary = "Create a namespace",
    description = "Creates a namespace inside an active project and initializes stats rows for existing target languages.",
    params(("project_public_id" = Uuid, Path, description = "Public UUID of the project that will own the namespace.")),
    request_body(content = NamespaceRequest, description = "Namespace attributes to create."),
    responses(
        (status = 201, description = "Namespace created.", body = Namespace),
        (status = 404, description = "Project was not found or has been deleted.", body = crate::error::ErrorBody),
        (status = 409, description = "An active namespace with the same name already exists in this project.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
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
    operation_id = "list_namespaces",
    summary = "List namespaces",
    description = "Returns active namespaces for one project ordered by namespace ID.",
    params(("project_public_id" = Uuid, Path, description = "Public UUID of the project whose namespaces should be listed.")),
    responses(
        (status = 200, description = "Active namespaces ordered by ID.", body = [Namespace]),
        (status = 404, description = "Project was not found or has been deleted.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
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
    operation_id = "get_namespace",
    summary = "Get a namespace",
    description = "Fetches one active namespace by numeric namespace ID within the project.",
    params(
        ("project_public_id" = Uuid, Path, description = "Public UUID of the project that owns the namespace."),
        ("namespace_id" = i64, Path, description = "Numeric namespace ID returned by the namespace list or create endpoint.")
    ),
    responses(
        (status = 200, description = "Namespace found.", body = Namespace),
        (status = 404, description = "Project or namespace was not found, or either has been deleted.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
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
    operation_id = "update_namespace",
    summary = "Update a namespace",
    description = "Renames an active namespace within the project.",
    params(
        ("project_public_id" = Uuid, Path, description = "Public UUID of the project that owns the namespace."),
        ("namespace_id" = i64, Path, description = "Numeric namespace ID returned by the namespace list or create endpoint.")
    ),
    request_body(content = NamespaceRequest, description = "Replacement namespace attributes."),
    responses(
        (status = 200, description = "Namespace updated.", body = Namespace),
        (status = 404, description = "Project or namespace was not found, or either has been deleted.", body = crate::error::ErrorBody),
        (status = 409, description = "An active namespace with the same name already exists in this project.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
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
    operation_id = "delete_namespace",
    summary = "Delete a namespace",
    description = "Soft-deletes a namespace. Deleting an unknown or already-deleted namespace is treated as a no-op once the project exists.",
    params(
        ("project_public_id" = Uuid, Path, description = "Public UUID of the project that owns the namespace."),
        ("namespace_id" = i64, Path, description = "Numeric namespace ID returned by the namespace list or create endpoint.")
    ),
    responses(
        (status = 204, description = "Namespace delete accepted."),
        (status = 404, description = "Project was not found or has been deleted.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
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

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    app::AppState,
    error::AppResult,
    models::{KeysetPage, SourceString},
};

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SourceStringRequest {
    pub identifier: String,
    pub value: String,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ListStringsQuery {
    pub after_id: Option<i64>,
    pub limit: Option<i64>,
}

#[utoipa::path(
    post,
    path = "/api/v1/projects/{project_public_id}/namespaces/{namespace_id}/strings",
    tag = "strings",
    params(
        ("project_public_id" = Uuid, Path, description = "Project public ID"),
        ("namespace_id" = i64, Path, description = "Namespace ID")
    ),
    request_body = SourceStringRequest,
    responses((status = 201, description = "Source string created", body = SourceString))
)]
pub async fn create_source_string(
    State(state): State<AppState>,
    Path((project_public_id, namespace_id)): Path<(Uuid, i64)>,
    Json(request): Json<SourceStringRequest>,
) -> AppResult<(StatusCode, Json<SourceString>)> {
    let source_string = state
        .services
        .source_strings
        .create_source_string(project_public_id, namespace_id, request.identifier, request.value)
        .await?;
    Ok((StatusCode::CREATED, Json(source_string)))
}

#[utoipa::path(
    get,
    path = "/api/v1/projects/{project_public_id}/namespaces/{namespace_id}/strings",
    tag = "strings",
    params(
        ("project_public_id" = Uuid, Path, description = "Project public ID"),
        ("namespace_id" = i64, Path, description = "Namespace ID"),
        ListStringsQuery
    ),
    responses((status = 200, description = "Source strings", body = [SourceString]))
)]
pub async fn list_source_strings(
    State(state): State<AppState>,
    Path((project_public_id, namespace_id)): Path<(Uuid, i64)>,
    Query(query): Query<ListStringsQuery>,
) -> AppResult<Json<Vec<SourceString>>> {
    Ok(Json(
        state
            .services
            .source_strings
            .list_source_strings(
                project_public_id,
                namespace_id,
                KeysetPage::new(query.after_id, query.limit),
            )
            .await?,
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/projects/{project_public_id}/strings/{string_id}",
    tag = "strings",
    params(
        ("project_public_id" = Uuid, Path, description = "Project public ID"),
        ("string_id" = i64, Path, description = "Source string ID")
    ),
    responses((status = 200, description = "Source string", body = SourceString))
)]
pub async fn get_source_string(
    State(state): State<AppState>,
    Path((project_public_id, string_id)): Path<(Uuid, i64)>,
) -> AppResult<Json<SourceString>> {
    Ok(Json(
        state
            .services
            .source_strings
            .get_source_string(project_public_id, string_id)
            .await?,
    ))
}

#[utoipa::path(
    put,
    path = "/api/v1/projects/{project_public_id}/strings/{string_id}",
    tag = "strings",
    params(
        ("project_public_id" = Uuid, Path, description = "Project public ID"),
        ("string_id" = i64, Path, description = "Source string ID")
    ),
    request_body = SourceStringRequest,
    responses((status = 200, description = "Source string updated", body = SourceString))
)]
pub async fn update_source_string(
    State(state): State<AppState>,
    Path((project_public_id, string_id)): Path<(Uuid, i64)>,
    Json(request): Json<SourceStringRequest>,
) -> AppResult<Json<SourceString>> {
    Ok(Json(
        state
            .services
            .source_strings
            .update_source_string(project_public_id, string_id, request.identifier, request.value)
            .await?,
    ))
}

#[utoipa::path(
    delete,
    path = "/api/v1/projects/{project_public_id}/strings/{string_id}",
    tag = "strings",
    params(
        ("project_public_id" = Uuid, Path, description = "Project public ID"),
        ("string_id" = i64, Path, description = "Source string ID")
    ),
    responses((status = 204, description = "Source string deleted"))
)]
pub async fn delete_source_string(
    State(state): State<AppState>,
    Path((project_public_id, string_id)): Path<(Uuid, i64)>,
) -> AppResult<StatusCode> {
    state
        .services
        .source_strings
        .delete_source_string(project_public_id, string_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

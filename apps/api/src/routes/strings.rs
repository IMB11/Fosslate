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
    /// Stable key for the source string within its namespace, for example `nav.home`.
    pub identifier: String,
    /// Source-language text that translators will translate.
    pub value: String,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ListStringsQuery {
    /// Return strings with IDs greater than this value. Omit for the first page.
    pub after_id: Option<i64>,
    /// Maximum number of strings to return. Defaults to 100 and is clamped to 1-500.
    pub limit: Option<i64>,
}

#[utoipa::path(
    post,
    path = "/api/v1/projects/{project_public_id}/namespaces/{namespace_id}/strings",
    tag = "strings",
    operation_id = "create_source_string",
    summary = "Create a source string",
    description = "Creates a source-language string inside an active namespace and refreshes namespace-language stats for existing target languages.",
    params(
        ("project_public_id" = Uuid, Path, description = "Public UUID of the project that owns the namespace."),
        ("namespace_id" = i64, Path, description = "Numeric namespace ID that will receive the source string.")
    ),
    request_body(content = SourceStringRequest, description = "Source string identifier and source-language value."),
    responses(
        (status = 201, description = "Source string created.", body = SourceString),
        (status = 404, description = "Project or namespace was not found, or either has been deleted.", body = crate::error::ErrorBody),
        (status = 409, description = "An active source string with the same identifier already exists in this namespace.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn create_source_string(
    State(state): State<AppState>,
    Path((project_public_id, namespace_id)): Path<(Uuid, i64)>,
    Json(request): Json<SourceStringRequest>,
) -> AppResult<(StatusCode, Json<SourceString>)> {
    let source_string = state
        .services
        .source_strings
        .create_source_string(
            project_public_id,
            namespace_id,
            request.identifier,
            request.value,
        )
        .await?;
    Ok((StatusCode::CREATED, Json(source_string)))
}

#[utoipa::path(
    get,
    path = "/api/v1/projects/{project_public_id}/namespaces/{namespace_id}/strings",
    tag = "strings",
    operation_id = "list_source_strings",
    summary = "List source strings",
    description = "Returns active source strings in one namespace using forward-only keyset pagination ordered by source string ID.",
    params(
        ("project_public_id" = Uuid, Path, description = "Public UUID of the project that owns the namespace."),
        ("namespace_id" = i64, Path, description = "Numeric namespace ID whose source strings should be listed."),
        ListStringsQuery
    ),
    responses(
        (status = 200, description = "Active source strings ordered by ID.", body = [SourceString]),
        (status = 404, description = "Project or namespace was not found, or either has been deleted.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
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
    operation_id = "get_source_string",
    summary = "Get a source string",
    description = "Fetches one active source string by numeric string ID within the project.",
    params(
        ("project_public_id" = Uuid, Path, description = "Public UUID of the project that owns the source string."),
        ("string_id" = i64, Path, description = "Numeric source string ID returned by the string list or create endpoint.")
    ),
    responses(
        (status = 200, description = "Source string found.", body = SourceString),
        (status = 404, description = "Project or source string was not found, or either has been deleted.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
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
    operation_id = "update_source_string",
    summary = "Update a source string",
    description = "Replaces the identifier and source-language value for an active source string.",
    params(
        ("project_public_id" = Uuid, Path, description = "Public UUID of the project that owns the source string."),
        ("string_id" = i64, Path, description = "Numeric source string ID returned by the string list or create endpoint.")
    ),
    request_body(content = SourceStringRequest, description = "Replacement source string fields."),
    responses(
        (status = 200, description = "Source string updated.", body = SourceString),
        (status = 404, description = "Project or source string was not found, or either has been deleted.", body = crate::error::ErrorBody),
        (status = 409, description = "An active source string with the same identifier already exists in this namespace.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
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
            .update_source_string(
                project_public_id,
                string_id,
                request.identifier,
                request.value,
            )
            .await?,
    ))
}

#[utoipa::path(
    delete,
    path = "/api/v1/projects/{project_public_id}/strings/{string_id}",
    tag = "strings",
    operation_id = "delete_source_string",
    summary = "Delete a source string",
    description = "Soft-deletes a source string and refreshes namespace-language stats for its namespace.",
    params(
        ("project_public_id" = Uuid, Path, description = "Public UUID of the project that owns the source string."),
        ("string_id" = i64, Path, description = "Numeric source string ID returned by the string list or create endpoint.")
    ),
    responses(
        (status = 204, description = "Source string deleted."),
        (status = 404, description = "Project or source string was not found, or either has been deleted.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
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

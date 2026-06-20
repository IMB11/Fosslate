use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{app::AppState, error::AppResult, models::Translation};

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateTranslationRequest {
    pub target_language_id: i64,
    pub author_user_id: i64,
    pub value: String,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ListTranslationsQuery {
    pub target_language_id: i64,
}

#[utoipa::path(
    post,
    path = "/api/v1/projects/{project_public_id}/strings/{string_id}/translations",
    tag = "translations",
    params(
        ("project_public_id" = Uuid, Path, description = "Project public ID"),
        ("string_id" = i64, Path, description = "Source string ID")
    ),
    request_body = CreateTranslationRequest,
    responses((status = 201, description = "Translation created", body = Translation))
)]
pub async fn create_translation(
    State(state): State<AppState>,
    Path((project_public_id, string_id)): Path<(Uuid, i64)>,
    Json(request): Json<CreateTranslationRequest>,
) -> AppResult<(StatusCode, Json<Translation>)> {
    let translation = state
        .services
        .translations
        .create_translation(
            project_public_id,
            string_id,
            request.target_language_id,
            request.author_user_id,
            request.value,
        )
        .await?;
    Ok((StatusCode::CREATED, Json(translation)))
}

#[utoipa::path(
    get,
    path = "/api/v1/projects/{project_public_id}/strings/{string_id}/translations",
    tag = "translations",
    params(
        ("project_public_id" = Uuid, Path, description = "Project public ID"),
        ("string_id" = i64, Path, description = "Source string ID"),
        ListTranslationsQuery
    ),
    responses((status = 200, description = "Translation candidates", body = [Translation]))
)]
pub async fn list_translations(
    State(state): State<AppState>,
    Path((project_public_id, string_id)): Path<(Uuid, i64)>,
    Query(query): Query<ListTranslationsQuery>,
) -> AppResult<Json<Vec<Translation>>> {
    Ok(Json(
        state
            .services
            .translations
            .list_translations(project_public_id, string_id, query.target_language_id)
            .await?,
    ))
}

#[utoipa::path(
    delete,
    path = "/api/v1/projects/{project_public_id}/translations/{translation_id}",
    tag = "translations",
    params(
        ("project_public_id" = Uuid, Path, description = "Project public ID"),
        ("translation_id" = i64, Path, description = "Translation ID")
    ),
    responses((status = 204, description = "Translation deleted"))
)]
pub async fn delete_translation(
    State(state): State<AppState>,
    Path((project_public_id, translation_id)): Path<(Uuid, i64)>,
) -> AppResult<StatusCode> {
    state
        .services
        .translations
        .delete_translation(project_public_id, translation_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    app::AppState,
    error::{AppError, AppResult},
    models::Translation,
    routes::auth::CurrentUser,
};

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateTranslationRequest {
    /// Target language ID returned by the project language endpoints.
    pub target_language_id: i64,
    /// User ID of the author submitting this candidate translation.
    pub author_user_id: i64,
    /// Candidate translation text in the target language.
    pub value: String,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ListTranslationsQuery {
    /// Target language ID whose candidates should be returned.
    pub target_language_id: i64,
}

#[utoipa::path(
    post,
    path = "/api/v1/projects/{project_public_id}/strings/{string_id}/translations",
    tag = "translations",
    operation_id = "create_translation",
    summary = "Create a translation candidate",
    description = "Creates a candidate translation for one source string and target language, then recalculates the current and best-rated translation projection.",
    params(
        ("project_public_id" = Uuid, Path, description = "Public UUID of the project that owns the source string."),
        ("string_id" = i64, Path, description = "Numeric source string ID that will receive the translation candidate.")
    ),
    request_body(content = CreateTranslationRequest, description = "Translation candidate details."),
    responses(
        (status = 201, description = "Translation candidate created.", body = Translation),
        (status = 404, description = "Project, source string, target language, or author user was not found, or a parent resource has been deleted.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn create_translation(
    State(state): State<AppState>,
    CurrentUser(current_user): CurrentUser,
    Path((project_public_id, string_id)): Path<(Uuid, i64)>,
    Json(request): Json<CreateTranslationRequest>,
) -> AppResult<(StatusCode, Json<Translation>)> {
    if current_user.id != request.author_user_id {
        return Err(AppError::Forbidden);
    }

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
    operation_id = "list_translations",
    summary = "List translation candidates",
    description = "Returns active candidate translations for one source string and target language ordered by rating score descending, then oldest candidate first.",
    params(
        ("project_public_id" = Uuid, Path, description = "Public UUID of the project that owns the source string."),
        ("string_id" = i64, Path, description = "Numeric source string ID whose translation candidates should be listed."),
        ListTranslationsQuery
    ),
    responses(
        (status = 200, description = "Active translation candidates ordered by rating.", body = [Translation]),
        (status = 404, description = "Project or source string was not found, or either has been deleted.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
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
    operation_id = "delete_translation",
    summary = "Delete a translation candidate",
    description = "Soft-deletes a candidate translation. If the deleted candidate was approved, its approval is removed, then current translation and stats projections are recalculated.",
    params(
        ("project_public_id" = Uuid, Path, description = "Public UUID of the project that owns the translation candidate."),
        ("translation_id" = i64, Path, description = "Numeric translation candidate ID returned by translation create or list endpoints.")
    ),
    responses(
        (status = 204, description = "Translation candidate deleted."),
        (status = 404, description = "Project or translation candidate was not found, or a parent resource has been deleted.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
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

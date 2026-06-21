use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::{app::AppState, error::AppResult, models::NamespaceLanguageStats};

#[utoipa::path(
    get,
    path = "/api/v1/projects/{project_public_id}/stats/namespaces",
    tag = "stats",
    operation_id = "list_namespace_language_stats",
    summary = "List namespace language stats",
    description = "Returns translation coverage stats for every active namespace and target language in one project.",
    params(("project_public_id" = Uuid, Path, description = "Public UUID of the project whose namespace-language stats should be listed.")),
    responses(
        (status = 200, description = "Namespace-language stats ordered by namespace and target language.", body = [NamespaceLanguageStats]),
        (status = 404, description = "Project was not found or has been deleted.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn list_namespace_language_stats(
    State(state): State<AppState>,
    Path(project_public_id): Path<Uuid>,
) -> AppResult<Json<Vec<NamespaceLanguageStats>>> {
    Ok(Json(
        state
            .services
            .stats
            .list_namespace_language_stats(project_public_id)
            .await?,
    ))
}

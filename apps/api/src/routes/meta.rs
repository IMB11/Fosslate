use axum::{Json, extract::State};
use serde::Serialize;

use crate::{app::AppState, db, error::AppResult};

#[derive(Serialize, utoipa::ToSchema)]
pub struct MetaResponse {
    /// Application name reported by the API process.
    #[schema(value_type = String)]
    app: &'static str,
    /// API package version compiled into the binary.
    #[schema(value_type = String)]
    version: &'static str,
    /// Current database connectivity status.
    database: DatabaseStatus,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct DatabaseStatus {
    /// Database status string. `ok` means a simple database query completed successfully.
    #[schema(value_type = String)]
    status: &'static str,
}

#[utoipa::path(
    get,
    path = "/api/v1/meta",
    tag = "meta",
    operation_id = "get_meta",
    summary = "Get API metadata",
    description = "Returns API build metadata and a lightweight database connectivity status.",
    responses(
        (status = 200, description = "Application metadata and dependency status.", body = MetaResponse),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn meta(State(state): State<AppState>) -> AppResult<Json<MetaResponse>> {
    Ok(Json(MetaResponse {
        app: state.app_name,
        version: state.version,
        database: DatabaseStatus {
            status: db::status(&state.db).await,
        },
    }))
}

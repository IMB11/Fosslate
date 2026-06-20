use axum::Json;
use serde::Serialize;

use crate::error::AppResult;

#[derive(Serialize, utoipa::ToSchema)]
pub struct HealthResponse {
    /// Health status for the API process.
    #[schema(value_type = String)]
    status: &'static str,
    /// Service identifier for callers that aggregate health checks.
    #[schema(value_type = String)]
    service: &'static str,
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    operation_id = "health",
    summary = "Check API health",
    description = "Returns a static process-level health response. This endpoint does not check database connectivity; use `/api/v1/meta` for database status.",
    responses(
        (status = 200, description = "API process is responding.", body = HealthResponse)
    )
)]
pub async fn health() -> AppResult<Json<HealthResponse>> {
    Ok(Json(HealthResponse {
        status: "ok",
        service: "fosslate-api",
    }))
}

use axum::Json;
use serde::Serialize;

use crate::error::AppResult;

#[derive(Serialize, utoipa::ToSchema)]
pub struct HealthResponse {
    #[schema(value_type = String)]
    status: &'static str,
    #[schema(value_type = String)]
    service: &'static str,
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "API health status", body = HealthResponse)
    )
)]
pub async fn health() -> AppResult<Json<HealthResponse>> {
    Ok(Json(HealthResponse {
        status: "ok",
        service: "fosslate-api",
    }))
}

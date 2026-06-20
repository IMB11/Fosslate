use axum::Json;
use serde::Serialize;

use crate::error::AppResult;

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

pub async fn health() -> AppResult<Json<HealthResponse>> {
    Ok(Json(HealthResponse {
        status: "ok",
        service: "fosslate-api",
    }))
}

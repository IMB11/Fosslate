use axum::{Json, extract::State};
use serde::Serialize;

use crate::{app::AppState, db, error::AppResult};

#[derive(Serialize, utoipa::ToSchema)]
pub struct MetaResponse {
    #[schema(value_type = String)]
    app: &'static str,
    #[schema(value_type = String)]
    version: &'static str,
    database: DatabaseStatus,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct DatabaseStatus {
    #[schema(value_type = String)]
    status: &'static str,
}

#[utoipa::path(
    get,
    path = "/api/v1/meta",
    tag = "meta",
    responses(
        (status = 200, description = "Application metadata and dependency status", body = MetaResponse),
        (status = 500, description = "Database request failed", body = crate::error::ErrorBody)
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

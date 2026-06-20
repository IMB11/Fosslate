use axum::{extract::State, Json};
use serde::Serialize;

use crate::{app::AppState, db, error::AppResult};

#[derive(Serialize)]
pub struct MetaResponse {
    app: &'static str,
    version: &'static str,
    database: DatabaseStatus,
}

#[derive(Serialize)]
pub struct DatabaseStatus {
    status: &'static str,
}

pub async fn meta(State(state): State<AppState>) -> AppResult<Json<MetaResponse>> {
    Ok(Json(MetaResponse {
        app: state.app_name,
        version: state.version,
        database: DatabaseStatus {
            status: db::status(&state.db).await,
        },
    }))
}

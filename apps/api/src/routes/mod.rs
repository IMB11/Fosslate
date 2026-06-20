use axum::{routing::get, Router};

use crate::app::AppState;

pub mod health;
pub mod meta;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health::health))
        .route("/api/v1/meta", get(meta::meta))
}


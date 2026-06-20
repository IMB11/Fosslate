use axum::{routing::get, Json, Router};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use crate::app::AppState;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::health::health,
        crate::routes::meta::meta,
    ),
    components(
        schemas(
            crate::error::ErrorBody,
            crate::routes::health::HealthResponse,
            crate::routes::meta::DatabaseStatus,
            crate::routes::meta::MetaResponse,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "meta", description = "Application metadata endpoints"),
    ),
    info(
        title = "Fosslate API",
        description = "HTTP API for Fosslate",
        version = env!("CARGO_PKG_VERSION"),
    )
)]
struct ApiDoc;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/openapi.json", get(openapi_json))
        .merge(Scalar::with_url("/docs", ApiDoc::openapi()))
}

async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

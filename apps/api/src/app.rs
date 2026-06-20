use axum::{
    http::{header, HeaderValue, Method},
    Router,
};
use sqlx::PgPool;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{config::Config, routes, services::Services};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub services: Services,
    pub app_name: &'static str,
    pub version: &'static str,
}

impl AppState {
    pub fn new(db: PgPool) -> Self {
        Self {
            services: Services::new(db.clone()),
            db,
            app_name: "Fosslate",
            version: env!("CARGO_PKG_VERSION"),
        }
    }
}

pub fn build(state: AppState, config: &Config) -> Router {
    let router = Router::new()
        .merge(routes::router())
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    match config.cors_allowed_origin.as_deref() {
        Some(origin) => match HeaderValue::from_str(origin) {
            Ok(origin) => router.layer(
                CorsLayer::new()
                    .allow_origin(origin)
                    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                    .allow_headers([header::CONTENT_TYPE]),
            ),
            Err(error) => {
                tracing::warn!(%error, origin, "ignoring invalid CORS_ALLOWED_ORIGIN");
                router
            }
        },
        None => router,
    }
}

use axum::{
    Router,
    http::{HeaderName, HeaderValue, Method, header},
    middleware,
};
use sqlx::PgPool;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{adapters::resend::EmailDeliveryClient, config::Config, routes, services::Services};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub services: Services,
    pub app_name: &'static str,
    pub version: &'static str,
}

impl AppState {
    pub fn new(db: PgPool, config: &Config, setup_secret: String, secrets_key: String) -> Self {
        Self::with_email_delivery(
            db,
            config,
            setup_secret,
            secrets_key,
            EmailDeliveryClient::resend(config.resend_api_url.clone()),
        )
    }

    pub fn with_email_delivery(
        db: PgPool,
        config: &Config,
        setup_secret: String,
        secrets_key: String,
        email_delivery: EmailDeliveryClient,
    ) -> Self {
        Self {
            services: Services::with_setup(
                db.clone(),
                setup_secret,
                config.public_app_url.clone(),
                config.public_api_url.clone(),
                secrets_key,
                email_delivery,
            ),
            db,
            app_name: "Fosslate",
            version: env!("CARGO_PKG_VERSION"),
        }
    }
}

pub fn build(state: AppState, config: &Config) -> Router {
    let middleware_state = state.clone();
    let router = Router::new()
        .merge(routes::router())
        .with_state(state)
        .layer(middleware::from_fn_with_state(
            middleware_state,
            routes::auth::auth_middleware,
        ))
        .layer(TraceLayer::new_for_http());

    match config.cors_allowed_origin.as_deref() {
        Some(origin) => match HeaderValue::from_str(origin) {
            Ok(origin) => router.layer(
                CorsLayer::new()
                    .allow_origin(origin)
                    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                    .allow_headers([
                        header::CONTENT_TYPE,
                        header::AUTHORIZATION,
                        HeaderName::from_static("x-csrf-token"),
                    ])
                    .allow_credentials(true),
            ),
            Err(error) => {
                tracing::warn!(%error, origin, "ignoring invalid CORS_ALLOWED_ORIGIN");
                router
            }
        },
        None => router,
    }
}

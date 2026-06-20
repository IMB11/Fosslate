use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("database request failed")]
    Database(#[from] sqlx::Error),
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct ErrorBody {
    #[schema(value_type = String)]
    error: &'static str,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::error!(error = %self, "request failed");

        let status = match self {
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (
            status,
            Json(ErrorBody {
                error: "internal_server_error",
            }),
        )
            .into_response()
    }
}

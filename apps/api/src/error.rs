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
    #[error("{0} not found")]
    NotFound(&'static str),
    #[error("{0}")]
    BadRequest(&'static str),
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct ErrorBody {
    #[schema(value_type = String)]
    pub error: &'static str,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::error!(error = %self, "request failed");

        let (status, error) = match self {
            Self::Database(sqlx::Error::RowNotFound) => (StatusCode::NOT_FOUND, "not_found"),
            Self::Database(sqlx::Error::Database(error)) if error.code().as_deref() == Some("23505") => {
                (StatusCode::CONFLICT, "conflict")
            }
            Self::Database(sqlx::Error::Database(error)) if error.code().as_deref() == Some("23503") => {
                (StatusCode::NOT_FOUND, "not_found")
            }
            Self::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_server_error"),
            Self::NotFound(_) => (StatusCode::NOT_FOUND, "not_found"),
            Self::BadRequest(_) => (StatusCode::BAD_REQUEST, "bad_request"),
        };

        (status, Json(ErrorBody { error })).into_response()
    }
}

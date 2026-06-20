use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;

use crate::{app::AppState, error::AppResult, models::User};

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateUserRequest {
    pub username: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/users",
    tag = "users",
    request_body = CreateUserRequest,
    responses((status = 201, description = "User created", body = User))
)]
pub async fn create_user(
    State(state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> AppResult<(StatusCode, Json<User>)> {
    let user = state.services.users.create_user(request.username).await?;
    Ok((StatusCode::CREATED, Json(user)))
}

#[utoipa::path(
    get,
    path = "/api/v1/users",
    tag = "users",
    responses((status = 200, description = "Users", body = [User]))
)]
pub async fn list_users(State(state): State<AppState>) -> AppResult<Json<Vec<User>>> {
    Ok(Json(state.services.users.list_users().await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/users/{user_id}",
    tag = "users",
    params(("user_id" = i64, Path, description = "User ID")),
    responses((status = 200, description = "User", body = User))
)]
pub async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
) -> AppResult<Json<User>> {
    Ok(Json(state.services.users.get_user(user_id).await?))
}

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;

use crate::{app::AppState, error::AppResult, models::User};

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateUserRequest {
    /// Unique username for the temporary user record. Leading and trailing whitespace is trimmed.
    pub username: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/users",
    tag = "users",
    operation_id = "create_user",
    summary = "Create a user",
    description = "Creates a temporary user account used as a translation author, voter, or reviewer.",
    request_body(content = CreateUserRequest, description = "User attributes to create."),
    responses(
        (status = 201, description = "User created.", body = User),
        (status = 409, description = "A user with the same username already exists.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
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
    operation_id = "list_users",
    summary = "List users",
    description = "Returns all temporary users ordered by user ID.",
    responses(
        (status = 200, description = "Users ordered by ID.", body = [User]),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn list_users(State(state): State<AppState>) -> AppResult<Json<Vec<User>>> {
    Ok(Json(state.services.users.list_users().await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/users/{user_id}",
    tag = "users",
    operation_id = "get_user",
    summary = "Get a user",
    description = "Fetches one temporary user by numeric user ID.",
    params(("user_id" = i64, Path, description = "Numeric user ID returned by user create or list endpoints.")),
    responses(
        (status = 200, description = "User found.", body = User),
        (status = 404, description = "User was not found.", body = crate::error::ErrorBody),
        (status = 500, description = "Database request failed.", body = crate::error::ErrorBody)
    )
)]
pub async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
) -> AppResult<Json<User>> {
    Ok(Json(state.services.users.get_user(user_id).await?))
}

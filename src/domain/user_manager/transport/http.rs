use axum::{
    extract::{Path, State}, http::StatusCode, Extension, Json
};
use utoipa::ToSchema;

use crate::{domain::{auth::entity::user::User,  user_manager::service::UserManagerError}, transport::http::auth::MiddlewareUserResponse, AppState};

/// Get user info
#[utoipa::path(
    get, 
    path = "/user/{user_id}", 
    params(
        ("user_id" = i32, Path, description = "ID of the user")
    ),
    tag = "short-link",
    responses(
        (status = 200, description = "OK", body = User),
        (status = 404, description = "Not Found"),
        (status = 500, description = "Internal Server Error"),)
)]
pub async fn get_user_info_get_handler(
    State(state): State<AppState>,
    Path(user_id): Path<i32>,
) -> Result<Json<User>, StatusCode> {
    match state.user_manager_service.get_user_info(user_id).await{
        Ok(user) => 
            Ok(Json(user)),
        Err(UserManagerError::UserNotFound(_)) => 
            Err(
                StatusCode::NOT_FOUND,
            ), 
        Err(_) => 
            Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct ChangeNameRequest{
    name: String,
}

/// Change user name
#[utoipa::path(
    post, 
    path = "/user/change-name", 
    tag = "short-link",
    request_body = ChangeNameRequest,
    responses(
        (status = 200, description = "OK"),
        (status = 500, description = "Internal Server Error"),)
)]

pub async fn change_name_post_handler(
    State(state): State<AppState>,
    Extension(middleware_user): Extension<MiddlewareUserResponse>,
    Json(payload): Json<ChangeNameRequest>, 
) -> Result<(), StatusCode> {
    match state.user_manager_service.change_name(middleware_user.user_id, payload.name).await{
        Ok(_) => 
            Ok(()),
        Err(_) => 
            Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
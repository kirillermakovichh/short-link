use axum::{
    extract::{Path, State}, http::StatusCode, Extension, response:: Redirect, Json
};
use utoipa::ToSchema;

use crate::{domain::link_manager::{entity::link::LinkId, service::LinkManagerError}, transport::http::auth::MiddlewareUserResponse, AppState};

/// View short link 
#[utoipa::path(
    get, 
    path = "/view/{link_id}", 
    params(
        ("link_id" = String, Path, description = "ID of the link", example = "SVa-")
    ),
    tag = "short-link",
    responses(
        (status = 200, description = "OK" ),
        (status = 404, description = "Not Found"),
        (status = 500, description = "Internal Server Error"),)
)]
pub async fn view_link_get_handler(
    State(state): State<AppState>,
    Path(link_id): Path<String>,
) -> Result<Redirect, StatusCode> {
    match state.link_manager_service.view_link(&LinkId::from_string(link_id)).await{
        Ok(link) => 
             Ok(Redirect::to(&link.redirect_url)),
        Err(LinkManagerError::LinkNotFound(_)) => 
            Err(
                StatusCode::NOT_FOUND,
            ), 
        Err(_) => 
            Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Get link views
#[utoipa::path(
    get, 
    path = "/get-views/{linkId}", 
    params(
        ("linkId" = String, Path, description = "ID of the link")
    ),
    tag = "short-link",
    responses(
        (status = 200, description = "OK", body = i64),
        (status = 404, description = "Not Found"),
        (status = 500, description = "Internal Server Error"),)
)]
pub async fn get_link_views_get_handler(
    State(state): State<AppState>,
    Path(link_id): Path<String>,
) -> Result<Json<i64>, StatusCode> {
    match state.link_manager_service.get_link_views(&LinkId::from_string(link_id)).await{
        Ok(views) => 
            Ok(Json(views)),
        Err(LinkManagerError::LinkNotFound(_)) => 
            Err(
                StatusCode::NOT_FOUND,
            ), 
        Err(_) => 
            Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct CreateLinkRequest{
    redirected_url: String,
    label: String
}

/// Create short link
#[utoipa::path(
    post, 
    path = "/create-link", 
    tag = "short-link",
    request_body = CreateLinkRequest,
    responses(
        (status = 200, description = "OK", body = LinkId),
        (status = 500, description = "Internal Server Error"),)
)]

pub async fn create_link_post_handler(
    State(state): State<AppState>,
    Extension(middleware_user): Extension<MiddlewareUserResponse>,
    Json(payload): Json<CreateLinkRequest>, 
) -> Result<Json<LinkId>, StatusCode> {
    match state.link_manager_service.create_link(middleware_user.user_id, payload. redirected_url,  payload.label).await{
        Ok(link_id) => 
            Ok(Json(link_id)),
        Err(_) => 
            Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Delete link
#[utoipa::path(
    delete, 
    path = "/delete-link/{linkId}", 
    params(
        ("linkId" = String, Path, description = "ID of the link")
    ),
    tag = "short-link",
    responses(
        (status = 200, description = "OK", body = bool),
        (status = 500, description = "Internal Server Error"),)
)]
pub async fn delete_link_delete_handler(
    State(state): State<AppState>,
    Extension(middleware_user): Extension<MiddlewareUserResponse>,
    Path(link_id): Path<String>,
) -> Result<Json<bool>, StatusCode> {
    match state.link_manager_service.delete_link(LinkId::from_string(link_id), middleware_user.user_id).await{
        Ok(_) => 
            Ok(Json(true)),
        Err(_) => 
            Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }

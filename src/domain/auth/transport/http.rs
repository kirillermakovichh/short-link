use crate::AppState;
use crate::domain::auth::service::AuthService;
use crate::tools::jwt::generate_jwt;
use crate::{
    domain::auth::infra::persistence::AuthPersistenceRepo, tools::password_hash::hash_password,
};

use axum::response::{IntoResponse, Response};
use axum::{Json, extract::State, http::StatusCode};
use axum_extra::extract::cookie::{Cookie, CookieJar};

use solar::trx_factory::SqlxTrxFactory;
use utoipa::ToSchema;

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct ErrorResponse {
    pub message: String,
}

#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct LoginResponse {
    user_id: i32,
    jwt: String,
}

#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct RegisterRequest {
    name: String,
    email: String,
    password: String,
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct RegisterResponse {
    user_id: i32,
}

/// Login
#[utoipa::path(
    post,
    path = "/login",
    request_body = LoginRequest,
    tag = "auth",
    responses(
        (status = 200, description = "OK", body = LoginResponse),
        (status = 401, description = "Unauthorized"),
    ),
)]
pub async fn login_post_handler(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<LoginRequest>,
) -> Result<(CookieJar, Json<LoginResponse>), (StatusCode, Json<ErrorResponse>)> {
    let hash_password = hash_password(&payload.password);

    match state.auth_service.login(payload.email, hash_password).await {
        Ok(user) => {
            // TODO: secret
            let jwt = match generate_jwt(user.id, "secret") {
                Ok(token) => token,
                Err(e) => {
                    let error_response = ErrorResponse {
                        message: format!("Failed to generate JWT: {}", e),
                    };
                    return Err((StatusCode::BAD_REQUEST, Json(error_response)));
                }
            };

            let cookie = Cookie::build(("token", jwt.clone()))
                .path("/")
                .http_only(true)
                .same_site(axum_extra::extract::cookie::SameSite::None)
                .build();

            let jar = jar.add(cookie);

            Ok((
                jar,
                Json(LoginResponse {
                    user_id: user.id,
                    jwt,
                }),
            ))
        }
        Err(error) => {
            let error_response = ErrorResponse {
                message: error.to_string(), // or any message you want
            };
            let response = (StatusCode::UNAUTHORIZED, Json(error_response));
            Err(response)
        }
    }
}

/// Registration
#[utoipa::path(
    post,
    path = "/register",
    request_body = RegisterRequest,
    tag = "auth",
    responses(
        (status = 200, description = "OK", body = RegisterResponse),
        (status = 400, description = "Bad Request"),
    ),
)]
pub async fn register_post_handler(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, StatusCode> {
    let hash_password = hash_password(&payload.password);

    match state
        .auth_service
        .register(payload.name, payload.email, hash_password)
        .await
    {
        Ok(user_id) => Ok(Json(RegisterResponse { user_id })),
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

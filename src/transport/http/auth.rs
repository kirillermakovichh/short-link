use std::env;

use crate::{
    AppState,
    tools::jwt::{decode_token, is_valid},
};
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::IntoResponse,
};
use axum_extra::extract::cookie::CookieJar;

#[derive(Debug, Clone)]
pub struct MiddlewareUserResponse {
    pub user_id: i32,
    pub email: String,
}

pub async fn user_middleware(
    State(state): State<AppState>,
    jar: CookieJar,
    mut req: Request,
    next: Next,
) -> impl IntoResponse {
    let cookie = match jar.get("token") {
        Some(c) => c,
        None => return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    };

    let secret_key = env::var("SECRET_JWT").expect("SECRET_JWT must be set");

    let token = match decode_token(cookie.value(), &secret_key) {
        Ok(t) => t,
        Err(_) => return (StatusCode::UNAUTHORIZED, "Invalid token").into_response(),
    };
    if !is_valid(&token.claims) {
        return (StatusCode::UNAUTHORIZED, "Token expired").into_response();
    }

    let user = match state.auth_service.get_user_info(token.claims.sub).await {
        Ok(u) => u,
        Err(_) => return (StatusCode::NOT_FOUND, "User not found").into_response(),
    };

    req.extensions_mut().insert(MiddlewareUserResponse {
        user_id: user.id,
        email: user.email,
    });

    next.run(req).await
}

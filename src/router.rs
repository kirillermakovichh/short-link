use crate::{
    AppState,
    domain::{
        auth::transport::http::{login_post_handler, register_post_handler},
        link_manager::transport::http::{
            create_link_post_handler, get_link_views_get_handler, view_link_get_handler,
        },
    },
    transport::http::auth::user_middleware,
};
use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{get, post},
};

use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::domain::auth::transport::http::login_post_handler,
        crate::domain::auth::transport::http::register_post_handler,
        crate::domain::link_manager::transport::http::view_link_get_handler,
        crate::domain::link_manager::transport::http::get_link_views_get_handler,
        crate::domain::link_manager::transport::http::create_link_post_handler,
    ),
        servers(
        (url = "http://localhost:3000", description = "Local server")
    ),

    tags((name = "short-link", description = "API Documentation")))]
struct ApiDoc {}

pub fn build_router(app_state: AppState) -> Router {
    Router::new()
        .route("/login", post(login_post_handler))
        .route("/register", post(register_post_handler))
        .route(
            "/create-link",
            post(create_link_post_handler)
                .route_layer(from_fn_with_state(app_state.clone(), user_middleware)),
        )
        .route(
            "/view/{link-id}",
            get(view_link_get_handler)
                .route_layer(from_fn_with_state(app_state.clone(), user_middleware)),
        )
        .route(
            "/get-views/{link-id}",
            get(get_link_views_get_handler)
                .route_layer(from_fn_with_state(app_state.clone(), user_middleware)),
        )
        .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi()))
        .with_state(app_state)
}

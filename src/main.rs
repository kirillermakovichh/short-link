pub mod config;
pub mod container;
pub mod domain;
pub mod tools;
pub mod transport;

use container::build_container;
use dotenv::dotenv;
use std::sync::Arc;

use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{get, post},
};
use domain::{
    auth::{
        infra::persistence::AuthPersistenceRepo,
        service::AuthService,
        transport::http::{login_post_handler, register_post_handler},
    },
    link_manager::{
        infra::persistence::LinkManagerPersistenceRepo,
        service::LinkManagerService,
        transport::http::{
            create_link_post_handler, get_link_views_get_handler, view_link_get_handler,
        },
    },
};
use solar::trx_factory::SqlxTrxFactory;
use transport::http::auth::user_middleware;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(Clone)]
pub struct AppState {
    auth_service: Arc<AuthService<AuthPersistenceRepo, SqlxTrxFactory>>,
    link_manager_service: Arc<LinkManagerService<LinkManagerPersistenceRepo, SqlxTrxFactory>>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let container = build_container().await;
    let app_state = AppState {
        auth_service: container.auth_service.clone(),
        link_manager_service: container.link_manager_service.clone(),
    };

    let addr = container.server_address.clone();

    let listener = tokio::net::TcpListener::bind(addr.clone())
        .await
        .expect("failed to bind to address");

    println!("Server running on: {addr:?}");

    axum::serve(listener, router(app_state)).await.unwrap();
}

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

fn router(app_state: AppState) -> Router {
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

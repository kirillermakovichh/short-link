pub mod domain;

use std::sync::Arc;

use axum::{
    Router,
    extract::Request,
    middleware::{Next, from_fn},
    response::IntoResponse,
    routing::{get, post},
};
use domain::link_manager::{
    infra::persistence::LinkManagerPersistenceRepo,
    service::LinkManagerService,
    transport::http::{
        create_link_post_handler, get_link_views_get_handler, view_link_get_handler,
    },
};
use eyre::Context;
use solar::trx_factory::SqlxTrxFactory;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
// use std::sync::Arc;

// use domain::auth::entity::user::UserId;
// use crate::domain::account::infra::persistence::AccountPersistenceRepo;
// use crate::domain::account::service::AccountService;

#[tokio::main]
async fn main() {
    let pool = sqlx::PgPool::connect("postgres://postgres:password@localhost:5432/short-link")
        .await
        .expect("failed to connect to db");

    let trx_factory = SqlxTrxFactory::new(pool);
    sqlx::migrate!("./migrations")
        .run(trx_factory.pool())
        .await
        .context("failed to run migrations")
        .unwrap();

    let link_manager_persistence_repo = LinkManagerPersistenceRepo::new(trx_factory.clone());

    let link_manager_service = Arc::new(LinkManagerService::new(
        link_manager_persistence_repo,
        trx_factory.clone(),
    ));

    let app_state = AppState {
        link_manager_service,
    };

    let addr = "127.0.0.1:3000";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind to address");
    println!("Server running on: {addr:?}");

    axum::serve(listener, router(app_state)).await.unwrap();
}

#[derive(Clone)]
pub struct AppState {
    link_manager_service: Arc<LinkManagerService<LinkManagerPersistenceRepo, SqlxTrxFactory>>,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::domain::link_manager::transport::http::view_link_get_handler,
        crate::domain::link_manager::transport::http::get_link_views_get_handler,
        crate::domain::link_manager::transport::http::create_link_post_handler,
    ),
    tags((name = "short-link", description = "API Documentation")))]
struct ApiDoc {}

fn router(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(console).route_layer(from_fn(middleware)))
        .route("/create-link", post(create_link_post_handler))
        .route("/view/{link-id}", get(view_link_get_handler))
        .route("/get-views/{link-id}", get(get_link_views_get_handler))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi()))
        .with_state(app_state)
}

async fn console() -> String {
    return ("HELLO qweWORLD").into();
}

async fn middleware(req: Request, next: Next) -> impl IntoResponse {
    println!("Hello fromqwemiddleware");

    next.run(req).await
}

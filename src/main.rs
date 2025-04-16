pub mod domain;

use axum::{response::IntoResponse, routing::{get, post}, Router};
use solar::trx_factory::SqlxTrxFactory;
use eyre::Context;
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

        // let account_persistence_repo = AccountPersistenceRepo::new(trx_factory.clone());

        // let account_service = Arc::new(AccountService::new(
        //     account_persistence_repo,
        //     trx_factory.clone(),
        // ));

        // let link_id = account_service.create_link(&UserId::generate(), "asdasd".to_string()).await.expect("failed to create link");
        // print!("link id: {link_id:?}");
        // account_service.view_link(&link_id).await.expect("failed to view link");
        // let views = account_service.get_link_views(&link_id).await.expect("failed to get link views");
        // println!("views: {views:?}");

    let addr = "127.0.0.1:3000";
    let listener = tokio::net::TcpListener::bind(addr).await.expect("failed to bind to address");
    println!("Server running on: {addr:?}");

    axum::serve(listener, router()).await.unwrap();
}

fn router() -> Router {
    Router::new()
    .route("/", get(hello_world))
    .route("/user", post(post_user))
}

async fn hello_world() -> &'static str {
    "Hello, World, from axum!"
}

async fn post_user() -> impl IntoResponse {
    "asasdd"
}

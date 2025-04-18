pub mod config;
pub mod container;
pub mod domain;
pub mod router;
pub mod tools;
pub mod transport;

use container::build_container;
use dotenv::dotenv;
use router::build_router;
use std::sync::Arc;

use domain::{
    auth::{infra::persistence::AuthPersistenceRepo, service::AuthService},
    link_manager::{infra::persistence::LinkManagerPersistenceRepo, service::LinkManagerService},
    user_manager::{infra::persistence::UserManagerPersistenceRepo, service::UserManagerService},
};
use solar::trx_factory::SqlxTrxFactory;

#[derive(Clone)]
pub struct AppState {
    auth_service: Arc<AuthService<AuthPersistenceRepo, SqlxTrxFactory>>,
    link_manager_service: Arc<LinkManagerService<LinkManagerPersistenceRepo, SqlxTrxFactory>>,
    user_manager_service: Arc<UserManagerService<UserManagerPersistenceRepo, SqlxTrxFactory>>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let container = build_container().await;

    let app_state = AppState {
        auth_service: container.auth_service.clone(),
        link_manager_service: container.link_manager_service.clone(),
        user_manager_service: container.user_manager_service.clone(),
    };

    let router = build_router(app_state);

    let addr = container.server_address.clone();

    let listener = tokio::net::TcpListener::bind(addr.clone())
        .await
        .expect("failed to bind to address");

    println!("Server running on: {addr:?}");

    axum::serve(listener, router).await.unwrap();
}

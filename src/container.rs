use eyre::Context;
use solar::trx_factory::SqlxTrxFactory;
use sqlx::{Pool, Postgres};
use std::sync::Arc;

use crate::{
    config::{ConfigSettings, load_config},
    domain::{
        auth::{infra::persistence::AuthPersistenceRepo, service::AuthService},
        link_manager::{
            infra::persistence::LinkManagerPersistenceRepo, service::LinkManagerService,
        },
    },
};

pub struct Container {
    pub config: ConfigSettings,
    pub pool: Pool<Postgres>,
    pub trx_factory: SqlxTrxFactory,
    pub auth_service: Arc<AuthService<AuthPersistenceRepo, SqlxTrxFactory>>,
    pub link_manager_service: Arc<LinkManagerService<LinkManagerPersistenceRepo, SqlxTrxFactory>>,
    pub server_address: String,
}

pub async fn build_container() -> Arc<Container> {
    let config = load_config().unwrap();

    let server_address = format!("{}:{}", config.server.host, config.server.port);

    let pool = sqlx::PgPool::connect(&config.database.url)
        .await
        .expect("failed to connect to db");

    let trx_factory = SqlxTrxFactory::new(pool.clone());
    sqlx::migrate!("./migrations")
        .run(trx_factory.pool())
        .await
        .context("failed to run migrations")
        .unwrap();

    let auth_persistence_repo = AuthPersistenceRepo::new(trx_factory.clone());
    let auth_service = Arc::new(AuthService::new(auth_persistence_repo, trx_factory.clone()));

    let link_manager_persistence_repo = LinkManagerPersistenceRepo::new(trx_factory.clone());
    let link_manager_service = Arc::new(LinkManagerService::new(
        link_manager_persistence_repo,
        trx_factory.clone(),
    ));

    Arc::new(Container {
        config,
        pool,
        trx_factory,
        auth_service,
        link_manager_service,
        server_address,
    })
}

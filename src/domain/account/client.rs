// use std::sync::Arc;

// use solar::trx_factory::SqlxTrxFactory;

// use super::entity::link::LinkId;
// use super::infra::persistence::AccountPersistenceRepo;
// use super::service::{self, AccountError};

// type AccountService =
//     service::AccountService<AccountPersistenceRepo, SqlxTrxFactory>;

// pub struct AccountClient {
//     account_service: Arc<AccountService, SqlxTrxFactory>,
// }

// impl AccountClient {
//     pub fn new(account_service: Arc<AccountService, SqlxTrxFactory>) -> Self {
//         Self {
//             account_service,
//         }
//     }

//     pub async fn view_link(&self, link_id: &LinkId) -> Result<Vec<i64>, AccountError> {
//         self.account_service
//             .view_link(link_id)
//             .await?
//     }

//     pub async fn get_link_views(&self, link_id: &LinkId) -> Result<Vec<i64>, AccountError> {
//         self.account_service
//             .get_link_views(link_id)
//             .await?
//     }
// }

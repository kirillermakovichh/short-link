use chrono::Utc;
use solar::trx_factory::{TrxContext, TrxFactory, TrxFactoryError};

use crate::domain::auth::entity::user::User;

use super::entity::user_no_password::UserNoPassword;

#[derive(thiserror::Error, Debug)]
pub enum PersistenceError {
    #[error("trx factory error: {0}")]
    TrxFactoryError(#[from] TrxFactoryError),
    #[error("internal error: {0:?}")]
    InternalError(#[from] eyre::Error),
}

#[async_trait::async_trait]
pub trait PersistenceRepo: Send + Sync {
    async fn save_user(&self, user: User, ctx: TrxContext) -> Result<i32, PersistenceError>;

    async fn get_user_by_id(
        &self,
        user_id: i32,
        ctx: TrxContext,
    ) -> Result<Option<User>, PersistenceError>;
}

#[derive(thiserror::Error, Debug)]
pub enum UserManagerError {
    #[error("trx factory error: {0}")]
    TrxFactoryError(#[from] TrxFactoryError),
    #[error("persistence error: {0}")]
    PersistenceError(#[from] PersistenceError),
    #[error("user not found: {0}")]
    UserNotFound(i32),
}

pub struct UserManagerService<P, T> {
    persistence_repo: P,
    trx_factory: T,
}

impl<P, T> UserManagerService<P, T>
where
    P: PersistenceRepo,
    T: TrxFactory,
{
    pub fn new(persistence_repo: P, trx_factory: T) -> Self {
        Self {
            persistence_repo,
            trx_factory,
        }
    }

    pub async fn change_name(&self, user_id: i32, name: String) -> Result<(), UserManagerError> {
        self.trx_factory
            .begin(async move |ctx| -> Result<(), UserManagerError> {
                let mut user = self
                    .persistence_repo
                    .get_user_by_id(user_id, ctx.clone())
                    .await?
                    .ok_or(UserManagerError::UserNotFound(user_id))?;

                user.name = name;
                user.updated_at = Utc::now();

                self.persistence_repo.save_user(user, ctx.clone()).await?;

                Ok(())
            })
            .await?;

        Ok(())
    }

    pub async fn get_user_info(&self, user_id: i32) -> Result<UserNoPassword, UserManagerError> {
        let user = self
            .persistence_repo
            .get_user_by_id(user_id, TrxContext::Empty)
            .await?
            .ok_or(UserManagerError::UserNotFound(user_id))?;

        Ok(UserNoPassword::new(user.id, user.name, user.email))
    }
}

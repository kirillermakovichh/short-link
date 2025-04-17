use solar::trx_factory::{TrxContext, TrxFactory, TrxFactoryError};

use super::entity::user::User;

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

    async fn login(
        &self,
        email: String,
        password: String,
        ctx: TrxContext,
    ) -> Result<User, PersistenceError>;
}

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("trx factory error: {0}")]
    TrxFactoryError(#[from] TrxFactoryError),
    #[error("persistence error: {0}")]
    PersistenceError(#[from] PersistenceError),
    #[error("incorrect email or password: {0}")]
    AuthenticationError(String),
}

pub struct AuthService<P, T> {
    persistence_repo: P,
    trx_factory: T,
}

impl<P, T> AuthService<P, T>
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

    pub async fn register(
        &self,
        name: String,
        email: String,
        password: String,
    ) -> Result<i32, AuthError> {
        let user_id = self
            .trx_factory
            .begin(async move |ctx| -> Result<i32, AuthError> {
                let user = User::new(name, email, password);
                let user_id = self
                    .persistence_repo
                    .save_user(user.clone(), ctx.clone())
                    .await?;

                Ok(user_id)
            })
            .await?;

        Ok(user_id)
    }

    pub async fn login(&self, email: String, password: String) -> Result<User, AuthError> {
        let user = self
            .trx_factory
            .begin(async move |ctx| -> Result<User, AuthError> {
                let user = self
                    .persistence_repo
                    .login(email, password, ctx.clone())
                    .await?;

                Ok(user)
            })
            .await?;

        Ok(user)
    }
}

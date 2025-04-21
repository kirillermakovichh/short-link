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
    ) -> Result<Option<User>, PersistenceError>;

    async fn get_user_by_email(
        &self,
        email: &str,
        ctx: TrxContext,
    ) -> Result<Option<User>, PersistenceError>;
}

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("trx factory error: {0}")]
    TrxFactoryError(#[from] TrxFactoryError),
    #[error("persistence error: {0}")]
    PersistenceError(#[from] PersistenceError),
    #[error("incorrect email or password")]
    IncorrectEmailOrPassword,
    #[error("user already exists")]
    UserAlreadyExists,
    #[error("user not found: {0:?}")]
    UserNotFound(i32),
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
                let existing_user = self
                    .persistence_repo
                    .get_user_by_email(&email, ctx.clone())
                    .await?;

                if existing_user.is_some() {
                    return Err(AuthError::UserAlreadyExists);
                }

                let user = User::new(name, email, password);
                let user_id = self.persistence_repo.save_user(user, ctx.clone()).await?;

                Ok(user_id)
            })
            .await?;

        Ok(user_id)
    }

    pub async fn login(&self, email: String, password: String) -> Result<User, AuthError> {
        let user = self
            .persistence_repo
            .login(email, password, TrxContext::Empty)
            .await?
            .ok_or(AuthError::IncorrectEmailOrPassword)?;

        Ok(user)
    }
}

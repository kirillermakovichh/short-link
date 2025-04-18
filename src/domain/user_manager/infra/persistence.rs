use chrono::Utc;
use eyre::Context;
use solar::trx_factory::{SqlxTrxFactory, TrxContext};

use crate::domain::auth::entity::user::User;
use crate::domain::user_manager::service::{PersistenceError, PersistenceRepo};

pub struct UserManagerPersistenceRepo {
    trx_factory: SqlxTrxFactory,
}

impl UserManagerPersistenceRepo {
    pub fn new(trx_factory: SqlxTrxFactory) -> Self {
        Self { trx_factory }
    }
}

#[async_trait::async_trait]
impl PersistenceRepo for UserManagerPersistenceRepo {
    async fn save_user(&self, user: User, ctx: TrxContext) -> Result<i32, PersistenceError> {
        let extract_or_create_trx = self.trx_factory.extract_or_create_trx(ctx).await?;
        let (trx, _) = extract_or_create_trx;
        let mut trx = trx.lock().await;
        let Some(trx) = trx.as_mut() else {
            return Err(PersistenceError::InternalError(eyre::eyre!(
                "failed to get sqlx transaction"
            )));
        };

        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (name, email, password, created_at, updated_at)
            VALUES ($1, $2, $3, NOW(), $4)
            ON CONFLICT (email) DO UPDATE
            SET name = EXCLUDED.name,
                updated_at = EXCLUDED.updated_at
            RETURNING id, email, name, password, created_at, updated_at
            "#,
            user.name,
            user.email,
            user.password,
            Utc::now(),
        )
        .fetch_one(&mut **trx)
        .await
        .context("failed to save user")?;

        Ok(user.id)
    }

    async fn get_user_by_id(
        &self,
        user_id: i32,
        ctx: TrxContext,
    ) -> Result<Option<User>, PersistenceError> {
        let extract_or_create_trx = self.trx_factory.extract_or_create_trx(ctx).await?;
        let (trx, _) = extract_or_create_trx;
        let mut trx = trx.lock().await;
        let Some(trx) = trx.as_mut() else {
            return Err(PersistenceError::InternalError(eyre::eyre!(
                "failed to get sqlx transaction"
            )));
        };

        let user_dto = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
            .fetch_optional(&mut **trx)
            .await
            .context("failed to find user by id")?;

        if let Some(user_dto) = user_dto {
            return Ok(Some(User::from(user_dto)));
        }

        Ok(None)
    }
}

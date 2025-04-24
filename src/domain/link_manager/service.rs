use redis::{AsyncCommands, RedisError, aio::ConnectionManager};
use serde_json::Error;
use solar::trx_factory::{TrxContext, TrxFactory, TrxFactoryError};

use super::entity::link::{Link, LinkId};

#[derive(thiserror::Error, Debug)]
pub enum PersistenceError {
    #[error("trx factory error: {0}")]
    TrxFactoryError(#[from] TrxFactoryError),
    #[error("internal error: {0:?}")]
    InternalError(#[from] eyre::Error),
}

#[async_trait::async_trait]
pub trait PersistenceRepo: Send + Sync {
    async fn save_link(&self, link: Link, ctx: TrxContext) -> Result<(), PersistenceError>;

    async fn increment_link_views(
        &self,
        link_id: &LinkId,
        ctx: TrxContext,
    ) -> Result<(), PersistenceError>;

    async fn next_link_id(&self, ctx: TrxContext) -> Result<LinkId, PersistenceError>;
    async fn find_link_by_id(
        &self,
        link_id: &LinkId,
        ctx: TrxContext,
    ) -> Result<Option<Link>, PersistenceError>;

    async fn delete_link(&self, link_id: LinkId, ctx: TrxContext) -> Result<(), PersistenceError>;
}

#[derive(thiserror::Error, Debug)]
pub enum LinkManagerError {
    #[error("trx factory error: {0}")]
    TrxFactoryError(#[from] TrxFactoryError),
    #[error("persistence error: {0}")]
    PersistenceError(#[from] PersistenceError),

    #[error("link not found: {0}")]
    LinkNotFound(LinkId),
    #[error("link not owned by user: {0}, {1}")]
    LinkNotOwnedByUser(LinkId, i32),

    #[error("failed to deserialize: {0}")]
    CacheError(Error),
}
pub struct LinkManagerService<P, T> {
    persistence_repo: P,
    trx_factory: T,
    redis_client: ConnectionManager,
    cache_expr_sec: u64,
}

impl<P, T> LinkManagerService<P, T>
where
    P: PersistenceRepo,
    T: TrxFactory,
{
    pub fn new(
        persistence_repo: P,
        trx_factory: T,
        redis_client: ConnectionManager,
        cache_expr_sec: u64,
    ) -> Self {
        Self {
            persistence_repo,
            trx_factory,
            redis_client,
            cache_expr_sec,
        }
    }

    pub async fn create_link(
        &self,
        user_id: i32,
        redirect_url: String,
        label: String,
    ) -> Result<LinkId, LinkManagerError> {
        let link: Link = self
            .trx_factory
            .begin(async move |ctx| -> Result<Link, LinkManagerError> {
                let link_id = self.persistence_repo.next_link_id(ctx.clone()).await?;
                let link = Link::new(link_id, user_id, redirect_url, label);
                self.persistence_repo
                    .save_link(link.clone(), ctx.clone())
                    .await?;

                Ok(link)
            })
            .await?;

        Ok(link.id.clone())
    }

    pub async fn view_link(&self, link_id: &LinkId) -> Result<Link, LinkManagerError> {
        let link = self
            .trx_factory
            .begin(async move |ctx| -> Result<Link, LinkManagerError> {
                let existing_link = self.get_and_cache_link(link_id, ctx.clone()).await?;

                self.persistence_repo
                    .increment_link_views(link_id, ctx.clone())
                    .await?;

                Ok(existing_link)
            })
            .await?;

        Ok(link)
    }

    async fn get_and_cache_link(
        &self,
        link_id: &LinkId,
        ctx: TrxContext,
    ) -> Result<Link, LinkManagerError> {
        let mut r = self.redis_client.clone();
        let string_link: Result<String, RedisError> = r.get(link_id.to_string()).await;

        if let Ok(s) = string_link {
            let link = serde_json::from_str(&s).map_err(|e| LinkManagerError::CacheError(e))?;

            return Ok(link);
        } else {
            let link = self
                .persistence_repo
                .find_link_by_id(link_id, ctx.clone())
                .await?
                .ok_or(LinkManagerError::LinkNotFound(link_id.clone()))?;

            if let Ok(serialized) = serde_json::to_string(&link) {
                let mut r_clone = self.redis_client.clone();
                let key = link_id.to_string();
                let expire = self.cache_expr_sec;

                tokio::spawn(async move {
                    let _: Result<(), RedisError> = r_clone.set_ex(key, serialized, expire).await;
                });
            }

            Ok(link)
        }
    }

    pub async fn get_link_views(&self, link_id: &LinkId) -> Result<i64, LinkManagerError> {
        let link = self
            .persistence_repo
            .find_link_by_id(link_id, TrxContext::Empty)
            .await?
            .ok_or(LinkManagerError::LinkNotFound(link_id.clone()))?;

        Ok(link.views)
    }

    pub async fn delete_link(&self, link_id: LinkId, user_id: i32) -> Result<(), LinkManagerError> {
        self.trx_factory
            .begin(async move |ctx| -> Result<(), LinkManagerError> {
                let link = self
                    .persistence_repo
                    .find_link_by_id(&link_id, TrxContext::Empty)
                    .await?
                    .ok_or(LinkManagerError::LinkNotFound(link_id.clone()))?;

                if link.user_id != user_id {
                    return Err(LinkManagerError::LinkNotOwnedByUser(
                        link_id.clone(),
                        user_id,
                    ));
                }

                self.persistence_repo
                    .delete_link(link_id, ctx.clone())
                    .await?;
                Ok(())
            })
            .await?;

        Ok(())
    }
}

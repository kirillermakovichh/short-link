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

    async fn next_link_id(&self, ctx: TrxContext) -> Result<LinkId, PersistenceError>;
    async fn find_link_by_id(
        &self,
        link_id: &LinkId,
        ctx: TrxContext,
    ) -> Result<Option<Link>, PersistenceError>;
}

#[derive(thiserror::Error, Debug)]
pub enum LinkManagerError {
    #[error("trx factory error: {0}")]
    TrxFactoryError(#[from] TrxFactoryError),
    #[error("persistence error: {0}")]
    PersistenceError(#[from] PersistenceError),

    #[error("link not found: {0}")]
    LinkNotFound(LinkId),
    #[error("link not owned by user: {0}")]
    LinkNotOwnedByUser(LinkId, i32),
}

pub struct LinkManagerService<P, T> {
    persistence_repo: P,
    trx_factory: T,
}

impl<P, T> LinkManagerService<P, T>
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
                let mut existing_link = self
                    .persistence_repo
                    .find_link_by_id(link_id, ctx.clone())
                    .await?
                    .ok_or(LinkManagerError::LinkNotFound(link_id.clone()))?;

                existing_link.increment_views();
                self.persistence_repo
                    .save_link(existing_link.clone(), ctx.clone())
                    .await?;

                Ok(existing_link)
            })
            .await?;

        Ok(link)
    }

    pub async fn get_link_views(&self, link_id: &LinkId) -> Result<i64, LinkManagerError> {
        let link = self
            .persistence_repo
            .find_link_by_id(link_id, TrxContext::Empty)
            .await?
            .ok_or(LinkManagerError::LinkNotFound(link_id.clone()))?;

        Ok(link.views)
    }
}

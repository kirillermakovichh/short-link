use solar::trx_factory::{TrxContext, SqlxTrxFactory};

use crate::domain::account::entity::link::{Link, LinkId};
use crate::domain::account::service::{PersistenceError, PersistenceRepo};

pub struct AccountPersistenceRepo {
    trx_factory: SqlxTrxFactory,
}

impl AccountPersistenceRepo {
    pub fn new(trx_factory: SqlxTrxFactory) -> Self {
        Self { trx_factory }
    }
}

#[derive(Debug)]
pub struct LinkDto {
    pub id: String,
    pub user_id: String,
    pub value: String,

    pub views: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_view: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<Link> for LinkDto{
    fn from(link: Link) -> Self{
        Self{
            id: link.id.value.to_string(),
            user_id: link.user_id.clone().to_string(),
            value: link.value.clone(),

            views: link.views.to_string(),
            created_at: link.created_at,
            last_view: link.last_view
        }
    }
}


#[async_trait::async_trait]
impl PersistenceRepo for AccountPersistenceRepo {
    async fn save_link(&self, link: Link, ctx: TrxContext) -> Result<Link, PersistenceError>{
        // todo!()
        Ok(link)
    }

    async fn next_link_id(&self, ctx: TrxContext) -> Result<LinkId, PersistenceError>{
        Ok(LinkId::generate())
    }
    async fn find_link_by_id(&self, link_id: LinkId, ctx: TrxContext) -> Result<Option<Link>, PersistenceError>{
        let (trx, _) = self.trx_factory.extract_or_create_trx(ctx).await?;
        let mut trx = trx.lock().await;
        let Some(trx) = trx.as_mut() else {
            return Err(PersistenceError::InternalError(eyre::eyre!(
                "failed to get sqlx transaction"
            )));
        };

        let link = sqlx::query_as!(
            LinkDto,
            "SELECT * FROM links WHERE id = $1",
            link_id.value.to_string()
        )
        .fetch_optional(&mut **trx)
        .await
        .context("failed to find link by id")?;

        let Some(link) = link else {
            return Ok(None);
        };


        Ok(None)
    }
}
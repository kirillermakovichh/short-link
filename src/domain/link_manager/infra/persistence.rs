use solar::trx_factory::{TrxContext, SqlxTrxFactory};
use eyre::Context;

use crate::domain::link_manager::entity::link::{Link, LinkId};
use crate::domain::link_manager::service::{PersistenceError, PersistenceRepo};
use crate::domain::auth::entity::user::UserId;

pub struct LinkManagerPersistenceRepo {
    trx_factory: SqlxTrxFactory,
}

impl LinkManagerPersistenceRepo {
    pub fn new(trx_factory: SqlxTrxFactory) -> Self {
        Self { trx_factory }
    }
}

#[derive(Debug)]
pub struct LinkDto {
    pub id: String,
    pub user_id: String,
    pub redirect_url: String,
    pub views: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_view: Option<chrono::DateTime<chrono::Utc>>,
}


impl From<Link> for LinkDto{
    fn from(link: Link) -> Self{
        Self{
            id: link.id.value.to_string(),
            user_id: link.user_id.clone().to_string(),
            redirect_url: link.redirect_url.clone(),

            views: link.views,
            created_at: link.created_at,
            last_view: link.last_view
        }
    }
}

impl From<LinkDto> for Link{
    fn from(link: LinkDto) -> Self{
        let id = LinkId::from_string(link.id);
        let user_id = UserId::try_from(link.user_id)
            .expect("Invalid user_id in LinkDto");


            Link::from_parts(
                id,
                user_id,
                link.redirect_url,
                link.views,
                link.created_at,
                link.last_view,
            )
    }
}


#[async_trait::async_trait]
impl PersistenceRepo for LinkManagerPersistenceRepo {
    async fn save_link(&self, link: Link, ctx: TrxContext) -> Result<(), PersistenceError>{
        let extract_or_create_trx = self.trx_factory.extract_or_create_trx(ctx).await?;
        let (trx, _) = extract_or_create_trx;
        let mut trx = trx.lock().await;
        let Some(trx) = trx.as_mut() else {
            return Err(PersistenceError::InternalError(eyre::eyre!(
                "failed to get sqlx transaction"
            )));
        };

        let link_dto = LinkDto::from(link.clone());
        sqlx::query!(
            r#"
            INSERT INTO links (id, user_id, redirect_url, views, created_at, last_view)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            link_dto.id,
            link_dto.user_id,
            link_dto.redirect_url,
            link_dto.views,
            link_dto.created_at,
            link_dto.last_view
        )
        .execute(&mut **trx)
        .await
        .context("failed to save link")?;

        Ok(())
    }

    async fn next_link_id(&self, ctx: TrxContext) -> Result<LinkId, PersistenceError>{
        let _= ctx;
        Ok(LinkId::generate())
    }
    async fn find_link_by_id(&self, link_id: LinkId, ctx: TrxContext) -> Result<Option<Link>, PersistenceError>{
        let extract_or_create_trx = self.trx_factory.extract_or_create_trx(ctx).await?;
        let (trx, _) = extract_or_create_trx;
        let mut trx = trx.lock().await;
        let Some(trx) = trx.as_mut() else {
            return Err(PersistenceError::InternalError(eyre::eyre!(
                "failed to get sqlx transaction"
            )));
        };

        let link_dto = sqlx::query_as!(
            LinkDto,
            r#"
            SELECT id, user_id, redirect_url, views, created_at, last_view
            FROM links
            WHERE id = $1
            "#,
            link_id.to_string()
        )
        .fetch_optional(&mut **trx)
        .await
        .context("failed to find link by id")?;

        if let Some(link_dto) = link_dto {
            return Ok(Some(Link::from(link_dto)));
        }

        Ok(None)
    }
}
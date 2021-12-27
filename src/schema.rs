use ammonia::clean;
use async_graphql::*;
use chrono::{DateTime, Utc};

pub struct QueryRoot;
use crate::{
    domain::{comment::Comment, story::Story, Item},
    result::Result,
    store::Store,
};

#[Object]
impl QueryRoot {
    async fn top_items(&self, ctx: &Context<'_>, limit: Option<u32>) -> Result<Vec<Item>> {
        let store = ctx.data::<Store>()?;

        let limit = limit.unwrap_or(50);
        let limit = limit.min(50);
        let ids = store.get_top_stories().await?;

        let mut items = store
            .get_items(ids.clone().into_iter().take(limit as usize).collect())
            .await?;

        Ok(ids.into_iter().filter_map(|id| items.remove(&id)).collect())
    }

    async fn item_by_id(&self, ctx: &Context<'_>, id: u32) -> Result<Option<Item>> {
        let store = ctx.data::<Store>()?;
        store.get_item(id).await
    }
}

#[Object]
impl Story {
    async fn id(&self) -> &u32 {
        &self.id
    }

    async fn descendants(&self) -> &u32 {
        &self.descendants
    }

    async fn by(&self) -> &str {
        &self.by
    }

    async fn kids(&self) -> &Option<Vec<u32>> {
        &self.kids
    }

    async fn score(&self) -> &u32 {
        &self.score
    }

    async fn title(&self) -> &str {
        &self.title
    }

    async fn url(&self) -> &Option<String> {
        &self.url
    }

    async fn text(&self) -> &Option<String> {
        &self.text
    }

    async fn time(&self) -> &DateTime<Utc> {
        &self.time
    }

    async fn children(&self, ctx: &Context<'_>) -> Result<Vec<Item>> {
        let store = ctx.data::<Store>()?;
        let kids = self.kids.clone().unwrap_or_default();
        let mut items = store.get_items(kids.clone()).await?;

        Ok(kids
            .into_iter()
            .filter_map(|id| items.remove(&id))
            .collect())
    }

    async fn safe_text(&self) -> String {
        clean(&self.text.clone().unwrap_or("".into()))
    }

    async fn human_time(&self) -> String {
        chrono_humanize::HumanTime::from(self.time.clone()).to_string()
    }
}

#[Object]
impl Comment {
    async fn id(&self) -> &u32 {
        &self.id
    }

    async fn by(&self) -> &str {
        &self.by
    }

    async fn kids(&self) -> &Option<Vec<u32>> {
        &self.kids
    }

    async fn parent(&self) -> &u32 {
        &self.parent
    }

    async fn text(&self) -> &str {
        &self.text
    }

    async fn time(&self) -> &DateTime<Utc> {
        &self.time
    }

    async fn children(&self, ctx: &Context<'_>) -> Result<Vec<Item>> {
        let store = ctx.data::<Store>()?;
        let kids = self.kids.clone().unwrap_or_default();
        let mut items = store.get_items(kids.clone()).await?;

        Ok(kids
            .into_iter()
            .filter_map(|id| items.remove(&id))
            .collect())
    }

    async fn safe_text(&self) -> String {
        clean(&self.text)
    }

    async fn human_time(&self) -> String {
        chrono_humanize::HumanTime::from(self.time.clone()).to_string()
    }
}

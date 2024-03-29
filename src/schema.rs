use ammonia::clean;
use async_graphql::*;
use chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::SqlitePool;

pub struct QueryRoot;

use crate::{
    domain::{comment::Comment, job::Job, story::Story, Item},
    result::Result,
    store::Store,
};

#[Object]
impl QueryRoot {
    async fn top_items(&self, ctx: &Context<'_>, limit: Option<u32>) -> Result<Vec<Item>> {
        let store = ctx.data::<Store>()?;
        let pool = ctx.data::<SqlitePool>()?;

        // Get top items
        let ids = sqlx::query!(
            r#"
            SELECT 
                item_id 
            FROM 
                list
            WHERE
                key = 'top_stories'
            ORDER BY 
               ordering ASC
            "#,
        )
        .fetch_all(&*pool)
        .await?
        .into_iter()
        .map(|row| row.item_id as u32)
        .collect::<Vec<u32>>();

        load_many(&store, ids, limit).await
    }

    async fn ask_items(&self, ctx: &Context<'_>, limit: Option<u32>) -> Result<Vec<Item>> {
        let store = ctx.data::<Store>()?;
        let ids = store.get_ask_stories().await?;
        load_many(&store, ids, limit).await
    }

    async fn job_items(&self, ctx: &Context<'_>, limit: Option<u32>) -> Result<Vec<Item>> {
        let store = ctx.data::<Store>()?;
        let ids = store.get_job_stories().await?;
        load_many(&store, ids, limit).await
    }

    async fn best_items(&self, ctx: &Context<'_>, limit: Option<u32>) -> Result<Vec<Item>> {
        let store = ctx.data::<Store>()?;
        let ids = store.get_best_stories().await?;
        load_many(&store, ids, limit).await
    }

    async fn new_items(&self, ctx: &Context<'_>, limit: Option<u32>) -> Result<Vec<Item>> {
        let store = ctx.data::<Store>()?;
        let ids = store.get_new_stories().await?;
        load_many(&store, ids, limit).await
    }

    async fn show_items(&self, ctx: &Context<'_>, limit: Option<u32>) -> Result<Vec<Item>> {
        let store = ctx.data::<Store>()?;
        let ids = store.get_show_stories().await?;
        load_many(&store, ids, limit).await
    }

    async fn item_by_id(&self, ctx: &Context<'_>, id: u32) -> Result<Option<Item>> {
        let store = ctx.data::<Store>()?;
        store.get_item(id).await
    }

    async fn bookmarked_items(&self, ctx: &Context<'_>, _limit: Option<u32>) -> Result<Vec<Item>> {
        let store = ctx.data::<Store>()?;
        let pool = ctx.data::<SqlitePool>()?;

        // Get bookmarked ids
        let ids = sqlx::query!(
            r#"
            SELECT 
                item_id 
            FROM 
                bookmarked_item
            ORDER BY 
                created_at DESC;
            "#,
        )
        .fetch_all(&*pool)
        .await?
        .into_iter()
        .map(|row| row.item_id as u32)
        .collect::<Vec<u32>>();

        load_many(&store, ids, None).await
    }

    async fn stats(&self, ctx: &Context<'_>) -> Result<String> {
        let pool = ctx.data::<SqlitePool>()?;

        // Get bookmarked ids
        let (backfill_ptr,): (String,) = sqlx::query_as(
            r#"
            SELECT 
                value 
            FROM 
                config
            WHERE 
                key='backfill_ptr'
            "#,
        )
        .fetch_one(&*pool)
        .await?;

        Ok(backfill_ptr)
    }
}

async fn load_many(store: &Store, ids: Vec<u32>, limit: Option<u32>) -> Result<Vec<Item>> {
    let limit = limit.unwrap_or(50);
    let limit = limit.min(50);

    let mut items = store
        .get_items(ids.clone().into_iter().take(limit as usize).collect())
        .await?;

    Ok(ids.into_iter().filter_map(|id| items.remove(&id)).collect())
}

#[derive(SimpleObject)]
struct ItemMetric {
    item_id: i64,
    metric: String,
    value: i64,
    created_at: NaiveDateTime,
}

#[derive(SimpleObject)]
struct Stats {
    item_count: i64,
    min_item_id: i64,
    max_item_id: i64,
}

#[Object]
impl Story {
    async fn id(&self) -> &u32 {
        &self.id
    }

    async fn total_comment_count(&self) -> &u32 {
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

    async fn descendants(&self, ctx: &Context<'_>) -> Result<Vec<Item>> {
        let store = ctx.data::<Store>()?;
        let items = store.get_descendants(self.id).await?;

        Ok(items.into_iter().map(|(_, item)| item).collect())
    }

    async fn safe_text(&self) -> String {
        clean(&self.text.clone().unwrap_or("".into()))
    }

    async fn human_time(&self) -> String {
        chrono_humanize::HumanTime::from(self.time.clone()).to_string()
    }

    async fn rank(&self, ctx: &Context<'_>) -> Result<Vec<ItemMetric>> {
        let pool = ctx.data::<SqlitePool>()?;
        let metrics = sqlx::query_as!(
            ItemMetric,
            r#"
            SELECT 
                * 
            FROM 
                item_metric
            WHERE
                item_id = ?1
            ORDER BY 
                created_at DESC
            "#,
            self.id
        )
        .fetch_all(pool)
        .await?;

        Ok(metrics)
    }

    async fn is_bookmarked(&self, ctx: &Context<'_>) -> Result<bool> {
        let pool = ctx.data::<SqlitePool>()?;

        // Get bookmarked ids
        let is_bookmarked = sqlx::query!(
            r#"
            SELECT 
                item_id 
            FROM 
                bookmarked_item
            WHERE
                item_id = ?1
            "#,
            self.id
        )
        .fetch_optional(&*pool)
        .await?;

        Ok(is_bookmarked.is_some())
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

    async fn ancestors(&self, ctx: &Context<'_>) -> Result<Vec<Item>> {
        let store = ctx.data::<Store>()?;
        let items = store.get_ancestors(self.id).await?;

        Ok(items.into_iter().map(|(_, item)| item).collect())
    }

    async fn descendants(&self, ctx: &Context<'_>) -> Result<Vec<Item>> {
        let store = ctx.data::<Store>()?;
        let items = store.get_descendants(self.id).await?;

        Ok(items.into_iter().map(|(_, item)| item).collect())
    }

    async fn safe_text(&self) -> String {
        clean(&self.text)
    }

    async fn human_time(&self) -> String {
        chrono_humanize::HumanTime::from(self.time.clone()).to_string()
    }

    async fn is_bookmarked(&self, ctx: &Context<'_>) -> Result<bool> {
        let pool = ctx.data::<SqlitePool>()?;

        // Get bookmarked ids
        let is_bookmarked = sqlx::query!(
            r#"
            SELECT 
                item_id 
            FROM 
                bookmarked_item
            WHERE
                item_id = ?1
            "#,
            self.id
        )
        .fetch_optional(&*pool)
        .await?;

        Ok(is_bookmarked.is_some())
    }
}

#[Object]
impl Job {
    async fn id(&self) -> &u32 {
        &self.id
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

    async fn text(&self) -> String {
        self.text.clone().unwrap_or_default()
    }

    async fn time(&self) -> &DateTime<Utc> {
        &self.time
    }

    async fn safe_text(&self) -> String {
        clean(&self.text.clone().unwrap_or_default())
    }

    async fn human_time(&self) -> String {
        chrono_humanize::HumanTime::from(self.time.clone()).to_string()
    }
}

// Mutations
pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn bookmark_item(&self, ctx: &Context<'_>, item_id: u32) -> Result<Option<Item>> {
        let pool = ctx.data::<SqlitePool>()?;

        // Insert a bookmarked item
        let now = Utc::now();
        let _ = sqlx::query!(
            r#"
        INSERT INTO 
            bookmarked_item (item_id, user_id, created_at)
        VALUES
            (?1, "dan", ?2)
        "#,
            item_id,
            now
        )
        .execute(&*pool)
        .await?;

        let store = ctx.data::<Store>()?;
        store.get_item(item_id).await
    }

    async fn unbookmark_item(&self, ctx: &Context<'_>, item_id: u32) -> Result<Option<Item>> {
        let pool = ctx.data::<SqlitePool>()?;

        // Insert a bookmarked item
        let _ = sqlx::query!(
            r#"
            DELETE FROM 
                bookmarked_item 
            WHERE
                item_id = ?1
            AND
                user_id = "dan"
            "#,
            item_id,
        )
        .execute(&*pool)
        .await?;

        let store = ctx.data::<Store>()?;
        store.get_item(item_id).await
    }
}

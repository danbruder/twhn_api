use std::collections::HashMap;

use crate::{
    db,
    domain::{Item, Updates},
    hn_client::HnClient,
    result::{Error, Result},
};
use futures::{stream, StreamExt};
use sqlx::sqlite::SqlitePool;

#[derive(Clone)]
pub struct Store {
    client: HnClient,
    pool: SqlitePool,
}

impl Store {
    pub fn new(pool: SqlitePool) -> Self {
        let client = HnClient::new();

        Self { client, pool }
    }

    pub async fn get_item(&self, id: u32) -> Result<Option<Item>> {
        println!("GET {}", id);
        if let Some(item) = db::Item::load(id).fetch_optional(&self.pool).await? {
            return Ok(Some(item.into()));
        }

        self.get_and_store_item(id).await
    }

    pub async fn get_and_store_item(&self, id: u32) -> Result<Option<Item>> {
        if let Some(item) = self.client.get_item(id).await.ok().flatten() {
            // Store it
            let db_item: db::Item = item.clone().into();
            db_item.insert().execute(&self.pool).await?;

            Ok(Some(item))
        } else {
            Ok(None)
        }
    }

    pub async fn get_items(&self, ids: Vec<u32>) -> Result<HashMap<u32, Item>> {
        stream::iter(ids)
            .map(|id| async move { Ok::<_, Error>((id, self.get_item(id).await?)) })
            .buffer_unordered(500)
            .fold(Ok(HashMap::new()), |output, next| async {
                let mut output = output?;
                if let (id, Some(story)) = next? {
                    output.insert(id, story);
                }
                Ok(output)
            })
            .await
    }

    pub async fn get_and_store_items(&self, ids: Vec<u32>) -> Result<HashMap<u32, Item>> {
        let items = self.get_items(ids).await?;

        let mut tx = self.pool.begin().await?;
        for (_, item) in items.iter() {
            let db_item: db::Item = item.clone().into();
            db_item.insert().execute(&mut tx).await?;
        }

        tx.commit().await?;

        Ok(items)
    }

    pub async fn get_descendants(&self, id: u32) -> Result<HashMap<u32, Item>> {
        let mut results = HashMap::new();

        if let Some(item) = self.get_item(id).await? {
            let mut to_fetch = item.kids();

            while to_fetch.len() > 0 {
                let children = self.get_items(to_fetch.clone()).await?;
                to_fetch = vec![];
                for (id, child) in children.into_iter() {
                    to_fetch.extend(child.kids());
                    results.insert(id, child);
                }

                // fuse
                if results.len() > 10_000 {
                    break;
                }
            }
        }

        Ok(results)
    }

    pub async fn get_ancestors(&self, id: u32) -> Result<HashMap<u32, Item>> {
        let mut results = HashMap::new();

        if let Some(item) = self.get_item(id).await? {
            let mut to_fetch = item.parent();

            while let Some(parent_id) = to_fetch {
                to_fetch = None;
                if let Some(parent) = self.get_item(parent_id).await? {
                    to_fetch = parent.parent();
                    results.insert(parent_id, parent);
                }

                // fuse
                if results.len() > 10_000 {
                    break;
                }
            }
        }

        Ok(results)
    }

    pub async fn get_top_stories(&self) -> Result<Vec<u32>> {
        self.client.get_top_stories().await
    }

    pub async fn get_ask_stories(&self) -> Result<Vec<u32>> {
        self.client.get_ask_stories().await
    }

    pub async fn get_show_stories(&self) -> Result<Vec<u32>> {
        self.client.get_show_stories().await
    }

    pub async fn get_job_stories(&self) -> Result<Vec<u32>> {
        self.client.get_job_stories().await
    }

    pub async fn get_best_stories(&self) -> Result<Vec<u32>> {
        self.client.get_best_stories().await
    }

    pub async fn get_new_stories(&self) -> Result<Vec<u32>> {
        self.client.get_new_stories().await
    }

    pub async fn get_updates(&self) -> Result<Updates> {
        self.client.get_updates().await
    }

    pub async fn get_max_item_id(&self) -> Result<u32> {
        self.client.get_max_item_id().await
    }
}

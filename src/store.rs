use std::collections::HashMap;

use crate::{
    domain::Item,
    hn_client::HnClient,
    result::{Error, Result},
};
use dashmap::DashMap;
use futures::{stream, StreamExt};
use std::time::{Duration, SystemTime};

#[derive(Clone)]
pub struct Store {
    client: HnClient,
    item_cache: DashMap<u32, (Item, SystemTime)>,
}

impl Store {
    pub fn new() -> Self {
        let client = HnClient::new();

        Self {
            client,
            item_cache: DashMap::new(),
        }
    }

    pub async fn get_item(&self, id: u32) -> Result<Option<Item>> {
        // 5 minutes of caching
        let deadline = Duration::from_secs(60 * 5);
        let item = self
            .item_cache
            .get(&id)
            .as_deref()
            .map(|(item, time)| (item.clone(), time.elapsed().ok()));

        match item {
            // Poor man's eviction
            Some((_, Some(elapsed))) if elapsed >= deadline => self.get_and_cache_item(id).await,
            Some((_, None)) => self.get_and_cache_item(id).await,
            Some((item, _)) => Ok(Some(item.clone())),
            None => self.get_and_cache_item(id).await,
        }
    }

    async fn get_and_cache_item(&self, id: u32) -> Result<Option<Item>> {
        // Get a new one
        if let Some(item) = self.client.get_item(id).await.ok().flatten() {
            self.item_cache
                .insert(item.id(), (item.clone(), SystemTime::now()));
            // Store it
            Ok(Some(item))
        } else {
            // Clear cache
            self.item_cache.remove(&id);
            Ok(None)
        }
    }

    pub async fn get_items(&self, ids: Vec<u32>) -> Result<HashMap<u32, Item>> {
        stream::iter(ids)
            .map(|id| async move { Ok::<_, Error>((id, self.get_item(id).await?)) })
            .buffer_unordered(50)
            .fold(Ok(HashMap::new()), |output, next| async {
                let mut output = output?;
                if let (id, Some(story)) = next? {
                    output.insert(id, story);
                }
                Ok(output)
            })
            .await
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
}

use std::collections::HashMap;

use crate::{
    domain::Item,
    hn_client::HnClient,
    result::{Error, Result},
};
use dashmap::DashMap;
use futures::{stream, StreamExt};

pub struct Store {
    client: HnClient,
    item_cache: DashMap<u32, Item>,
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
        if let Some(item) = self.item_cache.get(&id) {
            Ok(Some(item.clone()))
        } else {
            if let Some(item) = self.client.get_item(id).await? {
                self.item_cache.insert(item.id(), item.clone());
                Ok(Some(item))
            } else {
                Ok(None)
            }
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
}

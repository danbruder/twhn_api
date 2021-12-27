use std::collections::HashMap;

use crate::{domain::Item, hn_client::HnClient, result::Result};
use sqlx::sqlite::SqlitePool;

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
        self.client.get_item(id).await
    }
    pub async fn get_items(&self, ids: Vec<u32>) -> Result<HashMap<u32, Item>> {
        self.client.get_items(ids).await
    }
    pub async fn get_top_stories(&self) -> Result<Vec<u32>> {
        self.client.get_top_stories().await
    }
    // pub async fn get_max_item_id(&self) -> Result<u32> {}
    // pub async fn get_new_stories(&self) -> Result<Vec<u32>> {}
    // pub async fn get_best_stories(&self) -> Result<Vec<u32>> {}
    // pub async fn get_ask_stories(&self) -> Result<Vec<u32>> {}
    // pub async fn get_show_stories(&self) -> Result<Vec<u32>> {}
    // pub async fn get_job_stories(&self) -> Result<Vec<u32>> {}
}

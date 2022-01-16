use std::time::Duration;

use crate::{domain::Item, result::Result};
use reqwest::{self, Client};

static API_BASE_URL: &str = "https://hacker-news.firebaseio.com/v0";

#[derive(Clone)]
pub struct HnClient {
    client: Client,
}

impl HnClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap();
        Self { client }
    }

    pub async fn get_item(&self, id: u32) -> Result<Option<Item>> {
        Ok(self
            .client
            .get(&format!("{}/item/{}.json", API_BASE_URL, id))
            .send()
            .await?
            .json()
            .await
            .ok())
    }

    pub async fn get_max_item_id(&self) -> Result<u32> {
        Ok(self
            .client
            .get(&format!("{}/maxitem.json", API_BASE_URL))
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn get_top_stories(&self) -> Result<Vec<u32>> {
        Ok(self
            .client
            .get(&format!("{}/topstories.json", API_BASE_URL))
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn get_new_stories(&self) -> Result<Vec<u32>> {
        Ok(self
            .client
            .get(&format!("{}/newstories.json", API_BASE_URL))
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn get_best_stories(&self) -> Result<Vec<u32>> {
        Ok(self
            .client
            .get(&format!("{}/beststories.json", API_BASE_URL))
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn get_ask_stories(&self) -> Result<Vec<u32>> {
        Ok(self
            .client
            .get(&format!("{}/askstories.json", API_BASE_URL))
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn get_show_stories(&self) -> Result<Vec<u32>> {
        Ok(self
            .client
            .get(&format!("{}/showstories.json", API_BASE_URL))
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn get_job_stories(&self) -> Result<Vec<u32>> {
        Ok(self
            .client
            .get(&format!("{}/jobstories.json", API_BASE_URL))
            .send()
            .await?
            .json()
            .await?)
    }
}

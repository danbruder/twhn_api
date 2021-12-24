use std::time::Duration;

use crate::{
    result::{Error, Result},
    types,
};
use futures::{stream, StreamExt};
use reqwest::{self, Client};
use std::collections::HashMap;

static API_BASE_URL: &str = "https://hacker-news.firebaseio.com/v0";

/// The API client.
pub struct HnClient {
    client: Client,
}

impl HnClient {
    /// Create a new `HnClient` instance.
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap();
        Self { client }
    }

    /// Return the item with the specified id.
    ///
    /// May return `None` if item id is invalid.
    pub async fn get_item(&self, id: u32) -> Result<Option<types::Item>> {
        Ok(self
            .client
            .get(&format!("{}/item/{}.json", API_BASE_URL, id))
            .send()
            .await?
            .json()
            .await
            .ok())
    }

    /// Return the items with the specified ids.
    ///
    /// May return `None` if item id is invalid.
    pub async fn get_items(&self, ids: Vec<u32>) -> Result<HashMap<u32, types::Item>> {
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

    /// Return the user with the specified username.
    ///
    /// May return `None` if username is invalid.
    pub async fn get_user(&self, username: &str) -> Result<Option<types::User>> {
        Ok(self
            .client
            .get(&format!("{}/user/{}.json", API_BASE_URL, username))
            .send()
            .await?
            .json()
            .await?)
    }

    /// Return the id of the newest item.
    ///
    /// To get the 10 latest items, you can decrement the id 10 times.
    pub async fn get_max_item_id(&self) -> Result<u32> {
        Ok(self
            .client
            .get(&format!("{}/maxitem.json", API_BASE_URL))
            .send()
            .await?
            .json()
            .await?)
    }

    /// Return a list of top story item ids.
    pub async fn get_top_stories(&self) -> Result<Vec<u32>> {
        Ok(self
            .client
            .get(&format!("{}/topstories.json", API_BASE_URL))
            .send()
            .await?
            .json()
            .await?)
    }

    /// Return a list of new story item ids.
    pub async fn get_new_stories(&self) -> Result<Vec<u32>> {
        Ok(self
            .client
            .get(&format!("{}/newstories.json", API_BASE_URL))
            .send()
            .await?
            .json()
            .await?)
    }

    /// Return a list of best story item ids.
    pub async fn get_best_stories(&self) -> Result<Vec<u32>> {
        Ok(self
            .client
            .get(&format!("{}/beststories.json", API_BASE_URL))
            .send()
            .await?
            .json()
            .await?)
    }

    /// Return up to 200 latest Ask HN story item ids.
    pub async fn get_ask_stories(&self) -> Result<Vec<u32>> {
        Ok(self
            .client
            .get(&format!("{}/askstories.json", API_BASE_URL))
            .send()
            .await?
            .json()
            .await?)
    }

    /// Return up to 200 latest Show HN story item ids.
    pub async fn get_show_stories(&self) -> Result<Vec<u32>> {
        Ok(self
            .client
            .get(&format!("{}/showstories.json", API_BASE_URL))
            .send()
            .await?
            .json()
            .await?)
    }

    /// Return up to 200 latest Job story item ids.
    pub async fn get_job_stories(&self) -> Result<Vec<u32>> {
        Ok(self
            .client
            .get(&format!("{}/jobstories.json", API_BASE_URL))
            .send()
            .await?
            .json()
            .await?)
    }

    /// Return a list of items and users that have been updated recently.
    pub async fn get_updates(&self) -> Result<types::Updates> {
        Ok(self
            .client
            .get(&format!("{}/updates.json", API_BASE_URL))
            .send()
            .await?
            .json()
            .await?)
    }
}

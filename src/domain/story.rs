use chrono::{serde::ts_seconds, DateTime, Utc};
use serde::Deserialize;

/// A story.
#[derive(Debug, Clone, Deserialize)]
pub struct Story {
    /// The item's unique id.
    pub id: u32,
    /// The total comment count.
    pub descendants: u32,
    /// The username of the item's author.
    pub by: String,
    /// The ids of the item's comments, in ranked display order.
    pub kids: Option<Vec<u32>>,
    /// The story's score.
    pub score: u32,
    /// The title of the story.
    pub title: String,
    /// The URL of the story.
    pub url: Option<String>,
    /// The story text. HTML.
    pub text: Option<String>,
    /// Creation date of the item, in Unix Time.
    #[serde(with = "ts_seconds")]
    pub time: DateTime<Utc>,
}

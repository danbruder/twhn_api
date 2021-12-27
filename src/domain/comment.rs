use chrono::{serde::ts_seconds, DateTime, Utc};
use serde::Deserialize;

/// A comment.
#[derive(Debug, Deserialize)]
pub struct Comment {
    /// The item's unique id.
    pub id: u32,
    /// The username of the item's author.
    pub by: String,
    /// The ids of the item's comments, in ranked display order.
    pub kids: Option<Vec<u32>>,
    /// The comment's parent: either another comment or the relevant story.
    pub parent: u32,
    /// The comment text. HTML.
    pub text: String,
    /// Creation date of the item, in Unix Time.
    #[serde(with = "ts_seconds")]
    pub time: DateTime<Utc>,
}

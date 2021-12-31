use chrono::{serde::ts_seconds, DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A comment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// The item's unique id.
    pub id: u32,
    /// The story's score, or the votes for a pollopt.
    pub score: u32,
    /// The job text. HTML.
    pub text: Option<String>,
    /// Creation date of the item, in Unix Time.
    #[serde(with = "ts_seconds")]
    pub time: DateTime<Utc>,
    /// The title of the job.
    pub title: String,
    /// The URL of the story.
    pub url: Option<String>,
}

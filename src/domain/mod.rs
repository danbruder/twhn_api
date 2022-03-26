use async_graphql::*;
use serde::{Deserialize, Serialize};

pub mod comment;
pub mod job;
pub mod story;
use comment::Comment;
use job::Job;
use story::Story;

/// An API item, for example a story or a comment.
#[derive(Debug, Clone, Deserialize, Serialize, Union)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Item {
    /// A story.
    Story(Story),
    /// A comment.
    Comment(Comment),
    /// A job.
    Job(Job),
}

impl Item {
    pub fn kids(&self) -> Vec<u32> {
        match self {
            Item::Story(story) => story.kids.clone().unwrap_or_default(),
            Item::Comment(comment) => comment.kids.clone().unwrap_or_default(),
            Item::Job(_) => vec![],
        }
    }

    pub fn parent(&self) -> Option<u32> {
        match self {
            Item::Comment(comment) => Some(comment.parent),
            _ => None,
        }
    }
}

/// A list of recently updated items and users.
#[derive(Debug, Deserialize)]
pub struct Updates {
    /// A list of recently changed items.
    pub items: Vec<u32>,
    /// A list of recently changed usernames.
    pub profiles: Vec<String>,
}

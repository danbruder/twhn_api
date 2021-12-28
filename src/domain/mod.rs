use async_graphql::*;
use serde::Deserialize;

pub mod comment;
pub mod story;
use comment::Comment;
use story::Story;

/// An API item, for example a story or a comment.
#[derive(Debug, Clone, Deserialize, Union)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Item {
    /// A story.
    Story(Story),
    /// A comment.
    Comment(Comment),
}

impl Item {
    pub fn id(&self) -> u32 {
        match self {
            Item::Story(story) => story.id,
            Item::Comment(comment) => comment.id,
        }
    }

    pub fn kids(&self) -> Vec<u32> {
        match self {
            Item::Story(story) => story.kids.clone().unwrap_or_default(),
            Item::Comment(comment) => comment.kids.clone().unwrap_or_default(),
        }
    }

    pub fn parent(&self) -> Option<u32> {
        match self {
            Item::Comment(comment) => Some(comment.parent),
            _ => None,
        }
    }
}

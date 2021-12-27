use async_graphql::*;
use serde::Deserialize;

pub mod comment;
pub mod story;
use comment::Comment;
use story::Story;

/// An API item, for example a story or a comment.
#[derive(Debug, Deserialize, Union)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Item {
    /// A story.
    Story(Story),
    /// A comment.
    Comment(Comment),
}

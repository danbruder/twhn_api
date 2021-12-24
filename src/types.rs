//! Item types returned by the API.

use crate::client::HnClient;
use async_graphql::*;
use serde::Deserialize;

/// An API item, for example a story or a comment.
#[derive(Debug, Deserialize, Union)]
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

/// A story.
#[derive(Debug, Deserialize, SimpleObject)]
#[graphql(complex)]
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
    pub time: u64,
}

#[ComplexObject]
impl Story {
    async fn comments(&self) -> Result<Vec<Comment>> {
        comments(self.kids.clone().unwrap_or_default()).await
    }
}

/// A comment.
#[derive(Debug, Deserialize, SimpleObject)]
#[graphql(complex)]
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
    pub time: u64,
}

#[ComplexObject]
impl Comment {
    async fn comments(&self) -> Result<Vec<Comment>> {
        comments(self.kids.clone().unwrap_or_default()).await
    }
}

async fn comments(ids: Vec<u32>) -> Result<Vec<Comment>> {
    let client = HnClient::new();
    let mut items = client.get_items(ids.clone()).await?;

    Ok(ids
        .into_iter()
        .filter_map(|id| items.remove(&id).and_then(|s| s.as_comment()))
        .collect())
}

/// A job.
#[derive(Debug, Deserialize, SimpleObject)]
pub struct Job {
    /// The item's unique id.
    pub id: u32,
    /// The story's score, or the votes for a pollopt.
    pub score: u32,
    /// The job text. HTML.
    pub text: Option<String>,
    /// Creation date of the item, in Unix Time.
    pub time: u64,
    /// The title of the job.
    pub title: String,
    /// The URL of the story.
    pub url: Option<String>,
}

/// A user profile.
#[derive(Debug, Deserialize, SimpleObject)]
pub struct User {
    /// The user's unique username. Case-sensitive.
    pub id: String,
    /// Creation date of the user, in Unix Time.
    pub created: u64,
    /// The user's karma.
    pub karma: u32,
    /// Delay in minutes between a comment's creation and its visibility to
    /// other users.
    pub delay: Option<u32>,
    /// The user's optional self-description. HTML.
    pub about: Option<String>,
    /// List of the user's stories, polls and comments.
    pub submitted: Vec<u32>,
}

/// A list of recently updated items and users.
#[derive(Debug, Deserialize, SimpleObject)]
pub struct Updates {
    /// A list of recently changed items.
    pub items: Vec<u32>,
    /// A list of recently changed usernames.
    pub profiles: Vec<String>,
}

impl Item {
    pub fn as_story(self) -> Option<Story> {
        match self {
            Item::Story(story) => Some(story),
            _ => None,
        }
    }

    pub fn as_comment(self) -> Option<Comment> {
        match self {
            Item::Comment(comment) => Some(comment),
            _ => None,
        }
    }
}

// impl Story {
//     pub fn url(&self) -> String {
//         match self.url {
//             Some(url) => url,
//             None => format!("https://news.ycombinator.com/item?id={}", self.id),
//         }
//     }
// }

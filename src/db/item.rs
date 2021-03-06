use crate::domain;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{
    query::{Query, QueryAs},
    sqlite::{Sqlite, SqliteArguments},
};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, sqlx::FromRow)]
pub struct Item {
    id: i64,
    original: String,
    descendants: Option<i64>,
    username: Option<String>,
    score: Option<i64>,
    title: Option<String>,
    url: Option<String>,
    body: Option<String>,
    time: Option<DateTime<Utc>>,
}

impl Item {
    pub fn load<'a>(id: u32) -> QueryAs<'a, Sqlite, Item, SqliteArguments<'a>> {
        sqlx::query_as::<Sqlite, Item>(
            r#"
            SELECT * FROM item
            WHERE id = ?1
            LIMIT 1
            "#,
        )
        .bind(id as i64)
    }

    pub fn insert<'a>(&'a self) -> Query<'a, Sqlite, SqliteArguments> {
        sqlx::query!(
            r#"
            INSERT OR REPLACE INTO item (id, original, descendants, username, score, title, url, body, time)
            VALUES 
            (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            self.id,
            self.original,
            self.descendants,
            self.username,
            self.score,
            self.title,
            self.url,
            self.body,
            self.time,
        )
    }
}

impl From<domain::Item> for Item {
    fn from(input: domain::Item) -> Self {
        let original = serde_json::to_string(&input).unwrap();
        match input {
            domain::Item::Story(inner) => Self {
                id: inner.id as i64,
                original,
                descendants: Some(inner.descendants as i64),
                username: Some(inner.by),
                score: Some(inner.score as i64),
                title: Some(inner.title),
                url: inner.url,
                body: inner.text,
                time: Some(inner.time),
            },
            domain::Item::Comment(inner) => Self {
                id: inner.id as i64,
                original,
                username: Some(inner.by),
                descendants: None,
                score: None,
                title: None,
                url: None,
                body: Some(inner.text),
                time: Some(inner.time),
            },
            domain::Item::Job(inner) => Self {
                id: inner.id as i64,
                original,
                username: None,
                descendants: None,
                score: Some(inner.score as i64),
                title: Some(inner.title),
                url: inner.url,
                body: inner.text,
                time: Some(inner.time),
            },
        }
    }
}

impl From<Item> for domain::Item {
    fn from(input: Item) -> Self {
        serde_json::from_str(&input.original).unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::Utc;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
    use std::str::FromStr;

    async fn setup() -> SqlitePool {
        let options = SqliteConnectOptions::from_str("sqlite::memory:").unwrap();
        let pool = SqlitePoolOptions::new().connect_lazy_with(options);
        sqlx::migrate!().run(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn inserts_item() {
        let pool = setup().await;
        let time = Utc::now().to_string();

        let item = Item {
            id: 1,
            original: "hey".into(),
            descendants: Some(2),
            username: Some("dan".into()),
            score: Some(3),
            title: Some("Title".into()),
            url: Some("https://dan.com".into()),
            body: Some("body".into()),
        };

        item.insert().execute(&pool).await.unwrap();

        let got = Item::load(1).fetch_one(&pool).await.unwrap();
        let want = item;
        assert_eq!(got, want);
    }
}

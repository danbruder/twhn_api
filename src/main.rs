use anyhow::Result;
use std::collections::HashMap;
use std::convert::Infallible;

use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema, SimpleObject};
use async_graphql_warp::{GraphQLBadRequest, GraphQLResponse};
use futures::{stream, StreamExt};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use warp::{http::Method, http::Response as HttpResponse, Filter, Rejection};

#[tokio::main]
async fn main() {
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();

    let graphql_post = async_graphql_warp::graphql(schema).and_then(
        |(schema, request): (
            Schema<QueryRoot, EmptyMutation, EmptySubscription>,
            async_graphql::Request,
        )| async move {
            Ok::<_, Infallible>(GraphQLResponse::from(schema.execute(request).await))
        },
    );

    let graphql_playground = warp::path::end().and(warp::get()).map(|| {
        HttpResponse::builder()
            .header("content-type", "text/html")
            .body(playground_source(GraphQLPlaygroundConfig::new("/")))
    });

    let cors = warp::cors()
        .allow_methods(&[Method::POST, Method::GET, Method::OPTIONS])
        .allow_credentials(true)
        .allow_headers(vec!["content-type", "X-Auth-Token", "X-Admin-Token"])
        .allow_any_origin();

    let routes = graphql_playground
        .or(graphql_post)
        .recover(|err: Rejection| async move {
            if let Some(GraphQLBadRequest(err)) = err.find() {
                return Ok::<_, Infallible>(warp::reply::with_status(
                    err.to_string(),
                    StatusCode::BAD_REQUEST,
                ));
            }

            Ok(warp::reply::with_status(
                "INTERNAL_SERVER_ERROR".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        })
        .with(cors);

    println!("Playground: http://localhost:8000");
    warp::serve(routes).run(([0, 0, 0, 0], 8000)).await;
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn version(&self) -> String {
        "1.0".into()
    }

    async fn top_stories(&self, limit: Option<usize>) -> Result<Vec<Story>> {
        let limit = limit.unwrap_or(50);
        let limit = limit.min(50);
        let ids = reqwest::get("https://hacker-news.firebaseio.com/v0/topstories.json")
            .await?
            .json::<Vec<i32>>()
            .await?;

        let mut stories = stream::iter(ids.clone())
            .take(limit as usize)
            .map(|id| async move {
                Ok::<_, reqwest::Error>((
                    id,
                    reqwest::get(format!(
                        "https://hacker-news.firebaseio.com/v0/item/{}.json",
                        id
                    ))
                    .await?
                    .json::<Story>()
                    .await?,
                ))
            })
            .buffer_unordered(50)
            .fold(Ok(HashMap::new()), |output, next| async {
                let mut output = output?;
                let (id, story) = next?;

                output.insert(id, story);
                Ok::<_, anyhow::Error>(output)
            })
            .await?;

        Ok(ids
            .into_iter()
            .filter_map(|id| stories.remove(&id))
            .collect())
    }
}

#[derive(Clone, Deserialize, Serialize, SimpleObject, Debug)]
struct Story {
    id: i32,
    title: String,
    url: Option<String>,
}

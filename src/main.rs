use std::convert::Infallible;

use ::http::StatusCode;
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::*;
use async_graphql_warp::{GraphQLBadRequest, GraphQLResponse};
use warp::{http::Method, http::Response as HttpResponse, Filter, Rejection};

#[allow(dead_code)]
mod client;
mod result;
#[allow(dead_code)]
mod types;

use client::HnClient;
use result::Result;
use types::*;

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

    async fn top_stories(&self, limit: Option<u32>) -> Result<Vec<Story>> {
        let client = HnClient::new();

        let limit = limit.unwrap_or(50);
        let limit = limit.min(50);
        let ids = client.get_top_stories().await?;

        let mut stories = client
            .get_items(ids.clone().into_iter().take(limit as usize).collect())
            .await?;

        Ok(ids
            .into_iter()
            .filter_map(|id| stories.remove(&id).and_then(|s| s.as_story()))
            .collect())
    }

    async fn story_by_id(&self, id: u32) -> Result<Option<Story>> {
        let client = HnClient::new();
        client
            .get_item(id)
            .await
            .map(|i| i.and_then(|j| j.as_story()))
    }

    async fn comment_by_id(&self, id: u32) -> Result<Option<Comment>> {
        let client = HnClient::new();
        client
            .get_item(id)
            .await
            .map(|i| i.and_then(|j| j.as_comment()))
    }
}

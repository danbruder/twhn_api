use std::convert::Infallible;
use std::env;
use std::str::FromStr;

use ::http::StatusCode;
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::*;
use async_graphql_warp::{GraphQLBadRequest, GraphQLResponse};
use dotenv::dotenv;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use warp::{http::Method, http::Response as HttpResponse, Filter, Rejection};

#[allow(dead_code)]
mod hn_client;

mod cron;
mod db;
mod domain;
mod result;
mod schema;
mod store;

use schema::{MutationRoot, QueryRoot};
use store::Store;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").unwrap_or("sqlite://data.db".to_string());

    let options = SqliteConnectOptions::from_str(&database_url)
        .unwrap()
        .create_if_missing(true);
    let pool = SqlitePoolOptions::new().connect_lazy_with(options);
    sqlx::migrate!().run(&pool).await.ok();

    let store = Store::new(pool.clone());
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(store.clone())
        .data(pool.clone())
        .finish();

    let graphql_post = async_graphql_warp::graphql(schema).and_then(
        |(schema, request): (
            Schema<QueryRoot, MutationRoot, EmptySubscription>,
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

    tokio::spawn(cron::start(store, pool));

    println!("Playground: http://localhost:8000");
    warp::serve(routes).run(([0, 0, 0, 0], 8000)).await;
}

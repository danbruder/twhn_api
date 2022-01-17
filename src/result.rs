//! Errors, type aliases, and functions related to working with `Result`.

use thiserror::Error;

/// Result
pub type Result<T> = std::result::Result<T, Error>;

/// Represents all the ways that the client can fail.
#[derive(Error, Debug)]
pub enum Error {
    /// ReqwestError
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error("query error")]
    GraphqlError(async_graphql::Error),
    #[error("database error")]
    DatabaseError(sqlx::Error),
}

impl From<async_graphql::Error> for Error {
    fn from(err: async_graphql::Error) -> Self {
        Error::GraphqlError(err)
    }
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Error::DatabaseError(err)
    }
}

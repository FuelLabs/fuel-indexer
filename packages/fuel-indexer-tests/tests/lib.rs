#[cfg(all(feature = "postgres"))]
mod graphql_api_postgres;

#[cfg(all(feature = "postgres"))]
mod indexing_postgres;

mod service;

#[cfg(all(feature = "postgres"))]
mod web_api_postgres;

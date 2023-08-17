#[cfg(all(feature = "postgres"))]
mod graphql_server;

#[cfg(all(feature = "postgres"))]
mod indexing;

mod service;

#[cfg(all(feature = "postgres"))]
mod web_server;

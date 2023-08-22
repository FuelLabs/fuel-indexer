#[cfg(all(feature = "postgres", not(feature = "trybuild")))]
mod graphql_server;

#[cfg(all(feature = "postgres", not(feature = "trybuild")))]
mod indexing;

#[cfg(not(feature = "trybuild"))]
mod service;

#[cfg(all(feature = "postgres", not(feature = "trybuild")))]
mod web_server;

#[cfg(feature = "trybuild")]
mod trybuild;

#![deny(unused_crate_dependencies)]

use serde::Deserialize;

pub mod api;
pub(crate) mod auth;
pub mod cli;
pub(crate) mod commands;
pub(crate) mod models;
mod uses;

#[derive(Debug, Deserialize)]
pub struct GraphQLQueryParams {
    introspect: bool,
}

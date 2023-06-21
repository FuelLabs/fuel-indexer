//! # fuel-indexer-lib
//!
//! `fuel-indexer-lib` is a collection of utilities used by the various `fuel-indexer-*` crates.

#![deny(unused_crate_dependencies)]

pub mod config;
pub mod defaults;
pub mod graphql;
pub mod manifest;
pub mod utils;

pub use fuel_indexer_types::type_id;

use proc_macro2::TokenStream;
use quote::quote;

/// The source of execution for the indexer.
#[derive(Default, Clone, Debug)]
pub enum ExecutionSource {
    /// The indexer is being executed as a standalone binary.
    Native,

    /// The indexer is being executed in a WASM runtime.
    #[default]
    Wasm,
}

impl ExecutionSource {
    pub fn async_awaitness(&self) -> (TokenStream, TokenStream) {
        match self {
            Self::Native => (quote! {async}, quote! {.await}),
            Self::Wasm => (quote! {}, quote! {}),
        }
    }
}

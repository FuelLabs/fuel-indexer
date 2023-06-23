//! # fuel-indexer-lib
//!
//! `fuel-indexer-lib` is a collection of utilities used by the various `fuel-indexer-*` crates.

#![deny(unused_crate_dependencies)]

pub mod config;
pub mod defaults;
pub mod manifest;
pub mod utils;

#[cfg(feature = "graphql")]
pub mod graphql;

use proc_macro2::TokenStream;
use quote::quote;
use sha2::{Digest, Sha256};

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

/// Derive a type ID from a namespace and given abstraction name.
pub fn type_id(namespace: &str, name: &str) -> i64 {
    // IMPORTANT: https://github.com/launchbadge/sqlx/issues/499
    let mut bytes = [0u8; 8];
    let digest = Sha256::digest(format!("{name}:{namespace}").as_bytes());
    bytes[..8].copy_from_slice(&digest[..8]);
    i64::from_be_bytes(bytes)
}

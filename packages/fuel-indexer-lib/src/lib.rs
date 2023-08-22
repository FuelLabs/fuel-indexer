//! # fuel_indexer_lib
//!
//! A collection of utilities used by the various `fuel-indexer-*` crates.

#![deny(unused_crate_dependencies)]
pub mod config;
pub mod defaults;
pub mod graphql;
pub mod manifest;
pub mod utils;

use proc_macro2::TokenStream;
use quote::quote;

/// Max size of Postgres array types.
pub const MAX_ARRAY_LENGTH: usize = 2500;

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

#[derive(Debug, Clone, Copy)]
pub enum WasmIndexerError {
    DeserializationError = 1,
    SerializationError,
    PutObjectError,
    UnableToSaveListType,
    UninitializedMemory,
    UnableToFetchLogString,
    GeneralError,
}

impl From<i32> for WasmIndexerError {
    fn from(value: i32) -> Self {
        match value {
            0 => unreachable!("WasmIndexerError index starts at 1"),
            1 => Self::DeserializationError,
            2 => Self::SerializationError,
            3 => Self::PutObjectError,
            4 => Self::UnableToSaveListType,
            5 => Self::UninitializedMemory,
            6 => Self::UnableToFetchLogString,
            _ => Self::GeneralError,
        }
    }
}

impl std::fmt::Display for WasmIndexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SerializationError => {
                write!(f, "Failed to serialize")
            }
            Self::DeserializationError => {
                write!(f, "Failed to deserialize")
            }
            Self::UnableToSaveListType => {
                write!(f, "Failed to save list")
            }
            Self::PutObjectError => {
                write!(f, "Failed to save object")
            }
            Self::UninitializedMemory => {
                write!(f, "Failed to create MemoryView for indexer")
            }
            Self::UnableToFetchLogString => {
                write!(f, "Failed to fetch log string")
            }
            Self::GeneralError => write!(f, "A WASM error occurred"),
        }
    }
}

impl std::error::Error for WasmIndexerError {}

/// Return a fully qualified indexer namespace.
pub fn fully_qualified_namespace(namespace: &str, identifier: &str) -> String {
    format!("{}_{}", namespace, identifier)
}

/// Return the name of the join table for the given entities.
pub fn join_table_name(a: &str, b: &str) -> String {
    format!("{}s_{}s", a, b)
}

/// Return the name of each TypeDefinition in the join table.
pub fn join_table_typedefs_name(join_table_name: &str) -> (String, String) {
    let mut parts = join_table_name.split('_');
    let a = parts.next().unwrap();
    let b = parts.next().unwrap();

    // Trim the plural 's' from the end of the TypeDefinition name.
    (a[0..a.len() - 1].to_string(), b[0..b.len() - 1].to_string())
}

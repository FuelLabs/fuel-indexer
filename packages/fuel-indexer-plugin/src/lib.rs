#![deny(unused_crate_dependencies)]

#[cfg(feature = "native-execution")]
pub mod native;
pub mod wasm;

use fuel_indexer_lib::{
    graphql::parser::{JoinTableMeta, JoinTableRelation, JoinTableRelationType},
    join_table_typedefs_name,
};

extern crate alloc;

/// Details for the many-to-many relationship.
///
/// This is essentially the same as `fuel_indexer_lib::graphql::parser::JoinTableRelation`, just
/// a bit more compile-time friendly.
#[derive(Debug, Clone)]
pub struct JoinMetadata<'a> {
    /// Name of join table.
    pub table_name: &'a str,

    /// Name of first column.
    pub parent_column_name: &'a str,

    pub parent_column_type: &'a str,

    /// Name of second column.
    pub child_column_name: &'a str,

    pub child_column_type: &'a str,

    /// Column positions
    pub child_position: usize,
}

impl<'a> JoinMetadata<'a> {
    pub fn table_name(&self) -> String {
        self.table_name.to_string()
    }

    pub fn parent_column_name(&self) -> String {
        self.parent_column_name.to_string()
    }

    pub fn parent_column_type(&self) -> String {
        self.parent_column_type.to_string()
    }

    pub fn parent_typedef_name(&self) -> String {
        join_table_typedefs_name(&self.table_name).0
    }

    pub fn child_column_name(&self) -> String {
        self.child_column_name.to_string()
    }

    pub fn child_column_type(&self) -> String {
        self.child_column_type.to_string()
    }

    pub fn child_typedef_name(&self) -> String {
        join_table_typedefs_name(&self.table_name).1
    }
}

pub mod types {
    pub use fuel_indexer_schema::FtColumn;
    pub use fuel_indexer_types::fuel::{BlockData, TxId};

    // Traits needed to access client type fields. Could also include this as a sub-module
    // of `fuel_indexer_types::fuel`.
    pub use fuel_indexer_types::fuel::field::*;
    pub use fuel_indexer_types::{fuel, prelude::*};

    // These imports are used in the indexer.rs module when iterating over
    // block transactions, in order to cache contract IDs.
    pub use std::collections::{HashMap, HashSet};
}

pub mod utils {
    pub use fuel_indexer_lib::utils::sha256_digest;
}

pub use bincode;
pub use fuel_indexer_lib::{
    graphql::MAX_FOREIGN_KEY_LIST_FIELDS,
    utils::{deserialize, serialize},
};

// Specifically we import `serde` here for the `Serialize` and `Deserialize` traits
// else the user would have to explicity import these in their indexer modules.
pub use serde;

// We import `serde_json` for the `From<T> for Json` in the `fuel-indexer-macro/schema` module.
pub use serde_json;

pub mod prelude {
    pub use super::{
        bincode, deserialize, serde, serde_json, serialize, types::*, utils::*,
        JoinMetadata, MAX_FOREIGN_KEY_LIST_FIELDS,
    };
    pub use crate::{debug, error, info, trace, warn};
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        Logger::error(&format!($($arg)*))
    }};
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        Logger::warn(&format!($($arg)*))
    }};
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        Logger::info(&format!($($arg)*))
    }};
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {{
        Logger::debug(&format!($($arg)*))
    }};
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {{
        Logger::trace(&format!($($arg)*))
    }};
}

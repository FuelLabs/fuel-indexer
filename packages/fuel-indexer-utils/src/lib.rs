//! # Fuel Indexer Utilities
//!
//! ## Quickstart: `prelude`
//!
//! You can quickly bootstrap an indexer by using types and traits from the `prelude` module.
//!
//! ```no_run
//! # #[allow(unused)]
//! use fuel_indexer_utils::prelude::*;
//! ```

/// Utility functions for Fuel indexers.
mod utilities;

pub use utilities::*;

/// Prelude for Fuel indexers.
pub mod prelude {
    pub use crate::utilities::*;
    pub use fuel_indexer_macros::indexer;
    pub use fuel_indexer_plugin::prelude::*;
}

/// Re-exported types and traits for Fuel indexers.
pub mod plugin {
    pub use fuel_indexer_plugin::*;
}

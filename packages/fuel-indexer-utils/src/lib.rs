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
//!
//! Examples on how you can use the prelude can be found in
//! the Hello World indexer example(https://fuellabs.github.io/fuel-indexer/master/examples/hello-world.html).

mod utilities;

pub mod prelude {
    pub use crate::utilities::*;
    pub use fuel_indexer_macros::indexer;
    pub use fuel_indexer_plugin::prelude::*;
}

pub mod plugin {
    pub use fuel_indexer_plugin::*;
}

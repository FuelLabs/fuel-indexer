#![allow(clippy::result_unit_err)]

pub mod schema;
pub mod spec;
pub mod store;
pub mod testing;

pub(self) mod self_prelude {
    pub use anyhow::anyhow;
    pub use async_trait::async_trait;
    pub use extension_trait::extension_trait;
}

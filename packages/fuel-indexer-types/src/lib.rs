#![deny(unused_crate_dependencies)]
pub mod ffi;
pub mod fuel;
pub mod receipt;
pub mod scalar;

pub use fuels::{
    core::codec::try_from_bytes,
    types::{
        bech32::{Bech32Address, Bech32ContractId},
        Bits256, Identity, SizedAsciiString,
    },
};

pub const FUEL_TYPES_NAMESPACE: &str = "fuel";

pub trait TypeId {
    fn type_id() -> usize;
}

pub mod prelude {
    pub use crate::ffi::*;
    pub use crate::fuel;
    pub use crate::receipt::*;
    pub use crate::scalar::*;
    pub use crate::{
        Bech32Address, Bech32ContractId, Bits256, Identity, SizedAsciiString, TypeId,
        FUEL_TYPES_NAMESPACE,
    };
}

pub mod ffi;
pub mod fuel;
pub mod indexer;
pub mod receipt;
pub mod scalar;

use sha2::{Digest, Sha256};

pub use fuels::{
    core::codec::try_from_bytes,
    types::{
        bech32::{Bech32Address, Bech32ContractId},
        Bits256, EvmAddress, Identity, SizedAsciiString, B512,
    },
};

pub const FUEL_TYPES_NAMESPACE: &str = "fuel";

/// Chain ID for the beta4 network.
pub const BETA4_CHAIN_ID: u64 = 0x00;

/// 256-bit alias.
#[allow(non_camel_case_types)]
pub type b256 = Bits256;

pub trait TypeId {
    fn type_id() -> usize;
}

pub mod prelude {
    pub use crate::ffi::*;
    pub use crate::fuel;
    pub use crate::indexer::*;
    pub use crate::receipt::*;
    pub use crate::scalar::*;
    pub use crate::{
        b256, type_id, Bech32Address, Bech32ContractId, Bits256, Identity,
        SizedAsciiString, TypeId, BETA4_CHAIN_ID, FUEL_TYPES_NAMESPACE,
    };
}

/// Derive a type ID from a namespace and given abstraction name.
pub fn type_id(namespace: &str, name: &str) -> i64 {
    // IMPORTANT: https://github.com/launchbadge/sqlx/issues/499
    let mut bytes = [0u8; 8];
    let digest = Sha256::digest(format!("{name}:{namespace}").as_bytes());
    bytes[..8].copy_from_slice(&digest[..8]);
    i64::from_be_bytes(bytes)
}

/// Utility functions for Fuel indexers.
use fuel_indexer_plugin::prelude::sha256_digest;
use fuel_indexer_plugin::types::{Bytes32, UID};

/// Returns the SHA256 hex digest of the input as a `SizedAsciiString`.
pub fn id(data: impl AsRef<[u8]>) -> UID {
    UID::new(sha256_digest(&data)).expect("Failed to create UID.")
}

/// Converts a given input into a `Bytes32` object by taking the first 32 bytes
/// of the input's SHA256 hex digest.
pub fn bytes32(data: impl AsRef<[u8]>) -> Bytes32 {
    let digest = sha256_digest(&data);
    let mut result = [0u8; 32];
    result.copy_from_slice(&digest.as_bytes()[0..32]);
    Bytes32::from(result)
}

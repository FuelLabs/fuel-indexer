/// Utility functions for Fuel indexers.
use fuel_indexer_plugin::prelude::sha256_digest;
use fuel_indexer_plugin::types::{Bytes32, SizedAsciiString};

/// Returns the SHA256 hex digest of the input as a `SizedAsciiString`.
pub fn id(data: impl AsRef<[u8]>) -> SizedAsciiString<64> {
    SizedAsciiString::<64>::new(sha256_digest(&data)).unwrap()
}

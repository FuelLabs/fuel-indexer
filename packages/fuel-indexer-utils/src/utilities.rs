/// Utility functions for Fuel indexers.
use fuel_indexer_plugin::prelude::sha256_digest;
use fuel_indexer_plugin::types::{Bytes32, SizedAsciiString};

/// Return a `u64` from a byte array.
pub fn u64_id(d: &[u8; 8]) -> u64 {
    u64::from_le_bytes(*d)
}

/// Returns the first eight bytes of data as a `u64`.
pub fn id8(data: impl AsRef<[u8]>) -> u64 {
    let data = sha256_digest(&data);
    let mut buff = [0u8; 8];
    buff.copy_from_slice(&data.as_bytes()[..8]);
    u64_id(&buff)
}

/// Returns the first thirty-two bytes of data as a `Bytes32`.
pub fn first32_bytes_to_bytes32(data: impl AsRef<[u8]>) -> Bytes32 {
    let data = sha256_digest(&data);
    let mut buff = [0u8; 32];
    buff.copy_from_slice(&data.as_bytes()[..32]);
    Bytes32::from(buff)
}

/// Returns a `u64` from a byte vector to be used as an ID for an entity record.
pub fn u64_id_from_inputs(id: &[u8; 32], inputs: Vec<u8>) -> u64 {
    let inputs = [id.to_vec(), inputs].concat();
    id8(inputs)
}

/// Returns a `Bytes32` from a byte vector.
pub fn bytes32_from_inputs(id: &[u8; 32], inputs: Vec<u8>) -> Bytes32 {
    let inputs = [id.to_vec(), inputs].concat();
    first32_bytes_to_bytes32(inputs)
}

/// Transform a `SizedAsciiString` into a `String` by trimming and truncating.
pub fn trim_sized_ascii_string<const LEN: usize>(s: &SizedAsciiString<LEN>) -> String {
    let mut s = s.to_string();
    let n = s.trim_end_matches(' ').len();
    s.truncate(n);
    s
}

/// Returns the thirty-two byte SHA256 hash of the input data as a `String`.
pub fn sha2id(data: impl AsRef<[u8]>) -> SizedAsciiString<64> {
    SizedAsciiString::<64>::new(sha256_digest(&data)).unwrap()
}

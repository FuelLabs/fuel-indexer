use crate::prelude::sha256_digest;
use crate::prelude::{Bytes32, SizedAsciiString};

pub fn u64_id(d: &[u8; 8]) -> u64 {
    u64::from_le_bytes(*d)
}

pub fn first8_bytes_to_u64(data: impl AsRef<[u8]>) -> u64 {
    let data = sha256_digest(&data);
    let mut buff = [0u8; 8];
    buff.copy_from_slice(&data.as_bytes()[..8]);
    u64_id(&buff)
}

pub fn first32_bytes_to_bytes32(data: impl AsRef<[u8]>) -> Bytes32 {
    let data = sha256_digest(&data);
    let mut buff = [0u8; 32];
    buff.copy_from_slice(&data.as_bytes()[..32]);
    Bytes32::from(buff)
}

pub fn u64_id_from_inputs(id: &[u8; 32], inputs: Vec<u8>) -> u64 {
    let inputs = [id.to_vec(), inputs].concat();
    first8_bytes_to_u64(inputs)
}

pub fn bytes32_from_inputs(id: &[u8; 32], inputs: Vec<u8>) -> Bytes32 {
    let inputs = [id.to_vec(), inputs].concat();
    first32_bytes_to_bytes32(inputs)
}

pub fn trim_sized_ascii_string<const LEN: usize>(s: &SizedAsciiString<LEN>) -> String {
    let mut s = s.to_string();
    let n = s.trim_end_matches(' ').len();
    s.truncate(n);
    s
}

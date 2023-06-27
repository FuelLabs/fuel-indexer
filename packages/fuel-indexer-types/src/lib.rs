#![deny(unused_crate_dependencies)]
pub mod db;
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
    pub use crate::{TypeId, FUEL_TYPES_NAMESPACE};
}

#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[test]
    fn test_into_json_blob_id() {
        let id: ID = 123;
        let as_json: Json = id.into();
        let as_bytes: Blob = id.into();

        assert_eq!(as_json, Json("123".to_string()));
        assert_eq!(as_bytes, Blob(vec![123, 0, 0, 0, 0, 0, 0, 0]));
    }

    #[test]
    fn test_into_json_blob_int4() {
        let int: Int4 = 42;
        let as_json: Json = int.into();
        let as_bytes: Blob = int.into();

        assert_eq!(as_json, Json("42".to_string()));
        assert_eq!(as_bytes, Blob(vec![42, 0, 0, 0]));
    }

    #[test]
    fn test_into_json_blob_int8() {
        let int: Int8 = 1234567890;
        let as_json: Json = int.into();
        let as_bytes: Blob = int.into();

        assert_eq!(as_json, Json("1234567890".to_string()));
        assert_eq!(as_bytes, Blob(vec![210, 2, 150, 73, 0, 0, 0, 0]));
    }

    #[test]
    fn test_into_json_blob_int16() {
        let int: Int16 = 123456789012345;
        let as_json: Json = int.into();
        let as_bytes: Blob = int.into();

        assert_eq!(as_json, Json("123456789012345".to_string()));
        assert_eq!(
            as_bytes,
            Blob(vec![
                121, 223, 13, 134, 72, 112, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ])
        );
    }

    #[test]
    fn test_into_json_blob_uint4() {
        let uint: UInt4 = 7;
        let as_json: Json = uint.into();
        let as_bytes: Blob = uint.into();

        assert_eq!(as_json, Json("7".to_string()));
        assert_eq!(as_bytes, Blob(vec![7, 0, 0, 0]));
    }

    #[test]
    fn test_into_json_blob_uint8() {
        let uint: UInt8 = 1234567890;
        let as_json: Json = uint.into();
        let as_bytes: Blob = uint.into();

        assert_eq!(as_json, Json("1234567890".to_string()));
        assert_eq!(as_bytes, Blob(vec![210, 2, 150, 73, 0, 0, 0, 0]));
    }

    #[test]
    fn test_into_json_blob_uint16() {
        let uint: UInt16 = 123456789012345;
        let as_json: Json = uint.into();
        let as_bytes: Blob = uint.into();

        assert_eq!(as_json, Json("123456789012345".to_string()));
        assert_eq!(
            as_bytes,
            Blob(vec![
                121, 223, 13, 134, 72, 112, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ])
        );
    }

    #[test]
    fn test_into_json_blob_timestamp() {
        let timestamp: Timestamp = 1234567890;
        let as_json: Json = timestamp.into();
        let as_bytes: Blob = timestamp.into();

        assert_eq!(as_json, Json("1234567890".to_string()));
        assert_eq!(as_bytes, Blob(vec![210, 2, 150, 73, 0, 0, 0, 0]));
    }
}

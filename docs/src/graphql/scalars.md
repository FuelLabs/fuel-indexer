# Scalars

The Fuel indexer has a collection of GraphQL scalars that cover virtually any value type in use on the Fuel network. The following list contains each GraphQL scalar type along with its equivalent Rust type.

| GraphQL Scalar | Rust Type | Notes |
--- | --- | --- 
| Address | `u8[32]` |
| AssetId | `u8[32]` |
| ContractId | `u8[32]` |
| Bytes4 | `u8[4]` |
| Bytes8 | `u8[8]` |
| Bytes32 | `u8[32]` |
| Bytes64 | `u8[64]` |
| Blob | `Vec<u8>` | Byte blob of arbitary size |
| BlockId | `u8[32]` | 32-byte block ID |
| Boolean | `bool` |
| Charfield | `String` | String of arbitrary size |
| HexString | `Vec<u8>` | Byte blob of arbitrary size |
| ID | `SizedAsciiString<64>` | Alias of `UID`
| Int1 | `i8` |
| Int4 | `i32` |
| Int8 | `i64` |
| Int16 | `i128` |
| Json | `String` | JSON string of arbitary size |
| MessageId | `u8[32]` |
| Nonce | `u8[32]` |
| Salt | `u8[32]` |
| Signature | `u8[64]` | 64-byte signature |
| Tai64Timestamp | `Tai64` | `Tai64` timestamp |
| Timestamp | `u64` |
| UInt1 | `u8` |
| UInt4 | `u32` |
| UInt8 | `u64` |
| UInt16 | `u128` |
| UID | `SizedAsciiString<64>` | 32-byte unique ID |
| Virtual | `String` | Used to store types tagged with `@virtual` directive |

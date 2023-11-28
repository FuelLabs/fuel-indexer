# Scalars

The Fuel indexer has a collection of GraphQL scalars that cover virtually any value type in use on the Fuel network. The following list contains each GraphQL scalar type along with its equivalent Rust type.

| GraphQL Scalar | Rust Type | Notes |
--- | --- | ---
| Address | `u8[32]` |
| AssetId | `u8[32]` |
| Boolean | `bool` |
| Bytes | `Vec<u8>` | Byte blob of arbitary size |
| Bytes32 | `u8[32]` |
| Bytes4 | `u8[4]` |
| Bytes64 | `u8[64]` |
| Bytes8 | `u8[8]` |
| ContractId | `u8[32]` |
| HexString | `Vec<u8>` | Byte blob of arbitrary size |
| I128 | `i128` |
| I32 | `i32` |
| I64 | `i64` |
| I16 | `i16` |
| I8 | `i8` |
| ID | `SizedAsciiString<64>` | Alias of `UID`
| Json | `String` | JSON string of arbitary size |
| U128 | `u128` |
| U32 | `u32` |
| U64 | `u64` |
| U8 | `u8` |
| UID | `SizedAsciiString<64>` | 32-byte unique ID |
| String | `String` | String of arbitrary size |

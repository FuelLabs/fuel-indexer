# Storing Info in a Database

The Fuel indexer uses [PostgreSQL](https://github.com/docker-library/postgres/blob/2f6878ca854713264ebb27c1ba8530c884bcbca5/14/bullseye/Dockerfile) as the primary database.

> 💡 We're open to supporting other storage solutions in the future.

## Data Types

Below is a mapping of GraphQL schema types to their Sway and database equivalents. Note that an empty cell denotes that there is no direct equivalent for the type in the corresponding domain.

| GraphQL Scalar | Sway Type | Postgres Type |
--- | --- | ---
| Address | `b256` | varchar(64) |
| AssetId | `u8[32]` | varchar(64) |
| Blob | `str[]` | varchar(10485760) |
| BlockId | | varchar(64) |
| Boolean | `bool` | boolean |
| Bytes4 | `str[4]` | varchar(8) |
| Bytes8 | `str[8]` | varchar(16) |
| Bytes32 | `str[32]` | varchar(64) |
| Bytes64 | `str[64]` | varchar(128) |
| Charfield | `str[]` | varchar(255) |
| ContractId | `b256` | varchar(64) |
| HexString | `str[]` | varchar(10485760) |
| ID | | varchar(64) primary key |
| Int1 | `u8` | integer |
| Int4 | `u32` | integer |
| Int8 | `u64` | bigint |
| Int16 | | numeric(39,0) |
| Json | `str[]` | json |
| MessageId | `str[32]` | varchar(64) |
| Nonce | `str[32]` | varchar(64) |
| Salt | `str[32]` | varchar(64) |
| Signature | `str[64]` | varchar(128) |
| Tai64Timestamp | | varchar(128) |
| Timestamp | `u64` | timestamp |
| UID | | varchar(64) |
| UInt1 | `u8` | integer |
| UInt4 | `u32` | integer |
| UInt8 | `u64` | numeric(20, 0) |
| UInt16 |  | numeric(39, 0) |
| Virtual | | json |

## Example

Let's define an `Event` struct in a Sway contract:

```sway
struct Event {
    id: u64,
    address: Address,
    block_height: u64,
}
```

The corresponding GraphQL schema to mirror this `Event` struct would resemble:

```graphql
type Event @entity {
    id: ID!
    account: Address!
    block_height: UInt8!
}
```

And finally, this GraphQL schema will generate the following Postgres schema:

```text
                                           Table "schema.event"
    Column   |     Type    | Collation | Nullable | Default | Storage  | Compression | Stats target | Description
--------------+-------------+-----------+----------+---------+----------+-------------+--------------+-------------
 id           |    bigint   |           | not null |         | plain    |             |              |
 block_height |    bigint   |           | not null |         | plain    |             |              |
 address      | varchar(64) |           | not null |         | plain    |             |              |
 object       |    bytea    |           | not null |         | extended |             |              |
Indexes:
    "event_pkey" PRIMARY KEY, btree (id)
Access method: heap
```

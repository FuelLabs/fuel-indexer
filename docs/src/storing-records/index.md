# Storing Info in a Database

The Fuel indexer uses [PostgreSQL](https://github.com/docker-library/postgres/blob/2f6878ca854713264ebb27c1ba8530c884bcbca5/14/bullseye/Dockerfile) as the primary database.

> 💡 We're open to supporting other storage solutions in the future.

## Data Types

Below is a mapping of GraphQL schema types to their Sway and database equivalents. Note that an empty cell denotes that there is no direct equivalent for the type in the corresponding domain.

| GraphQL Scalar | Sway Type | Postgres Type |
--- | --- | ---
| `Address` | `b256` | `varchar(64)` |
| `AssetId` | `u8[32]` | `varchar(64)` |
| `Boolean` | `bool` | `boolean` |
| `Bytes` | `str[]` | `varchar(10485760)` |
| `Bytes32` | `str[32]` | `varchar(64)` |
| `Bytes4` | `str[4]` | `varchar(8)` |
| `Bytes64` | `str[64]` | `varchar(128)` |
| `Bytes8` | `str[8]` | `varchar(16)` |
| `ContractId` | `b256` | `varchar(64)` |
| `I128` | | `numeric(39,0)` |
| `I32` | `u32` | `integer` |
| `I64` | `u64` | `bigint` |
| `I8` | `u8` | `integer` |
| `ID` | | `varchar(64) primary key` |
| `Json` | `str[]` | `json` |
| `U128` |  | `numeric(39, 0)` |
| `U32` | `u32` | `integer` |
| `U64` | `u64` | `numeric(20, 0)` |
| `U8` | `u8` | `integer` |
| `UID` | | `varchar(64)` |
| `String` | `str[]` | `varchar(255)` |

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
    block_height: U64!
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

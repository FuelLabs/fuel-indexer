# Types

Below is a mapping of GraphQL schema types to their Postgres equivalents, referencing [Postgres 14](https://www.postgresql.org/docs/14/datatype.html) data types. 

| Sway Type | GraphQL Schema Type | Postgres Type |
|------|----------|----------|
| u64 | ID | bigint primary key |
| b256 | Address | varchar(64) |
| str[4] | Bytes4 | varchar(16) |
| str[8] | Bytes8 | varchar(64) |
| str[32] | Bytes32 | varchar(64) |
| str[32] | AssetId | varchar(64) |
| b256 | ContractId | varchar(64) |
| str[32] | Salt | varchar(64) |
| u32 | Int4 | integer |
| u64 | Int8 | bigint |
| u64 | Timestamp | timestamp |
| str[] | Blob | bytes |

## Example

Let's define an `Event` struct in a Sway contract:

```sway
struct Event {
    address: Address,
    id: u64,
    block_height: u64,
}
```

Our GraphQL schema should resemble:

```code
type Event {
    id: ID!
    account: Address! @indexed
    block_height: Int8! @indexed
}
```

This will generate the following Postgres schema:

```code
                                           Table "schema.event"
  Column   |     Type    | Collation | Nullable | Default | Storage  | Compression | Stats target | Description 
-----------+-------------+-----------+----------+---------+----------+-------------+--------------+-------------
 id        |    bigint   |           | not null |         | plain    |             |              | 
 count     |    bigint   |           | not null |         | plain    |             |              | 
 address   | varchar(64) |           | not null |         | plain    |             |              | 
 object    |    bytea    |           | not null |         | extended |             |              | 
Indexes:
    "count_pkey" PRIMARY KEY, btree (id)
Access method: heap
```

The source code for these types can be found [here](https://github.com/FuelLabs/fuel-indexer/blob/master/schema/src/db/models.rs#L146).


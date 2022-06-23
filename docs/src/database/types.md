# Types

Below is a mapping of GraphQL schema types to their Postgres equivalents, referencing [Postgres 14](https://www.postgresql.org/docs/14/datatype.html) data types. 

| GraphQL Schema Type | Postgres Type |
|----------|----------|
| ID | bigint primary key |
| Address | varchar(64) |
| Bytes4 | varchar(16) |
| Bytes8 | varchar(64) |
| Bytes32 | varchar(64) |
| AssetId | varchar(64) |
| ContractId | varchar(64) |
| Salt | varchar(64) |
| Int4 | integer |
| Int8 | bigint |
| Timestamp | timestamp |
| Blob | bytes |

So for example if you've defined the following struct in your Sway contract (as well as the ABI JSON the contract generates)

```sway
struct Event {
    address: Address,
    id: u64,
    block_height: u64,
}
```

Your subsequenty GraphQL schema would resemble:

```code
type Event {
    id: ID!
    account: Address! @indexed
    block_height: Int8! @indexed
}
```

Which would then generate the following Postgres schema

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

The source code for these types can be found [here](https://github.com/FuelLabs/fuel-indexer/blob/master/schema/src/db/models.rs#L146)


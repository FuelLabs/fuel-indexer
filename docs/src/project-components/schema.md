# GraphQL Schema

The GraphQL schema is a required component of the Fuel indexer. When data is indexed into the database, the actual values that are persisted to the database will be values created using the data structures defined in the schema.

Below is a sample GraphQL schema for a Fuel indexer.

```graphql
type Metadata @entity(virtual: true) {
    imageUrl: Charfield!
    data: Blob
}

type Account @entity {
    id: ID!
    address: Address!
    index: UInt8!
    metadata: Metadata
}

type Wallet @entity {
    id: ID!
    name: Charfield!
    accounts: [Account!]!
}
```

For a complete list of all scalars that can be used in a Fuel indexer, please see the [GraphQL Scalars](./../graphql/scalars.md) section.

Further, for a complete list of how Sway data types, GraphQL scalar types, and Fuel indexer database types map to each other, please see the [Database Types](./../database/index.md) section.


Finally, for a more in-depth explanation on the schema being used above 👆🏽, please read the [GraphQL](./../graphql/index.md) section.

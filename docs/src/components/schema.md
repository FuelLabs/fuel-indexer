# GraphQL Schema

The GraphQL schema is a required component of the Fuel Indexer. When data is indexed and placed into the database, it will be done through the use of data structures defined in the schema.

In its most basic form, a schema should have a `schema` definition that contains a defined query type. The rest of the implementation is up to you. Here's an example of a well-formed schema:

```graphql
schema {
    query: QueryRoot
}

type QueryRoot {
    thing1: FirstThing
    thing2: SecondThing
}

type FirstThing {
    id: ID!
    value: UInt8!
}

type SecondThing {
    id: ID!
    other_value: UInt8!
    timestamp: Timestamp!
}
```

You should also familiarize yourself with the information under the [Database](database/index.md) section in order to ensure that data from your Sway contract is stored in the database as intended.

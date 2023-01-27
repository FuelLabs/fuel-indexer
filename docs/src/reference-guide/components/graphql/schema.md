# GraphQL Schema

The GraphQL schema is a required component of the Fuel indexer. When data is indexed into the database, the actual values that are persisted to the database will be values created using the data structures defined in the schema.

In its most basic form, a Fuel indexer GraphQL schema should have a `schema` definition that contains a defined query root. The rest of the implementation is up to you. Here's an example of a well-formed schema:

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

The types you see above (e.g., `ID`, `UInt8`, etc) are Fuel abstractions that were created to more seamlessly integrate with the Fuel VM and are not native to GraphQL. A deeper explanation on these
types can be found in [the Types section](../../data-types/types.md).

> Important: It is up to developers to manage their own unique IDs for each type, meaning that a data structure's `ID` field needs to be manually generated prior to saving it to the database. This generation can be as simple or complex as you want in order to fit your particular situation; the only requirement is that the developer implement their own custom generation. Examples can be found in the [Block Explorer](../../../examples/block-explorer.md) and [Hello World](../../../examples/hello-world.md) sections.

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

The types you see above (e.g., `ID`, `UInt8`, etc) are Fuel abstractions that were created to more seamlessly integrate with the Fuel VM. A deeper explanation on these
types can be found in [the Types section](../database/types.md).

> Important: These GraphQL schema types are not native to GraphQL.

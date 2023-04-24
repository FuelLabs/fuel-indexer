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
    optional_value: UInt8
    timestamp: Timestamp!
}
```

The types you see above (e.g., `ID`, `UInt8`, etc) are Fuel abstractions that were created to more seamlessly integrate with the Fuel VM and are not native to GraphQL. A deeper explanation on these
types can be found in [the Types section](../../data-types/types.md).

> Important: It is up to developers to manage their own unique IDs for each type, meaning that a data structure's `ID` field needs to be manually generated prior to saving it to the database. This generation can be as simple or complex as you want in order to fit your particular situation; the only requirement is that the developer implement their own custom generation. Examples can be found in the [Block Explorer](../../../examples/block-explorer.md) and [Hello World](../../../examples/hello-world.md) sections.

## Required and Optional Fields

Required fields are denoted with a `!` following its type; for example, the `value` field of the `FirstThing` type is a `UInt8` and is required to be present for the indexer to successfully persist the entity. If a certain piece of information is essential to your use case, then you should mark that field as required.

In contrast, optional fields are not required to be present for the indexer to persist the entity in storage. You can denote an optional field by just using the type name; for example, the `optional_value` field of the `SecondThing` type is optional, and should be a `UInt8` if present. If it's possible that a value might not always exist in the data you wish to index, consider making that the corresponding field optional. In your indexer code, you will need to use the `Option` Rust type when assigning a value to an optional field; values that are present should be assigned after being wrapped in `Some(..)` while absent values should be assigned using `None`.

> Important: The `ID` field is _always_ required. An indexer **will** return an error if an optional value is used for the `ID` field.

## Introspection
Introspection is a powerful feature in GraphQL that allows you to query the schema and retrieve its structure, types, and fields. Read more about introspection in the [GraphQL Docs](https://graphql.org/learn/introspection/)

To perform an introspection query, you can use the special `__schema` field, which will return information about the schema's types and fields. Here's an example introspection query that retrieves information about the types and their fields in the schema:

```graphql
query IntrospectionQuery {
  __schema {
    types {
      name
      fields {
        name
        type {
          name
          kind
        }
      }
    }
  }
}

```

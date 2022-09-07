# Foreign Keys

The Fuel Indexer service supports Foreign Key constraints and relationships using a combination of GraphQL schema and a data layer (whether Postgres or SQLite). Below you'll find a simple example of how to use Foreign Keys.

> IMPORTANT: At the moment, [due to some SQLite quirks](https://www.sqlite.org/omitted.html), Fuel Indexer SQLite only supports foreign key _relationships_, not foreign key _constraints_. We are very much open to changing this in the future.


To demonstrate how the Indexer uses GraphQL schema to resolve foreign key relationships, given the following schema:

```graphql
schema {
    query: QueryRoot
}

type QueryRoot {
    book: Book
    library: Library
}

type Book {
    id: ID!
    name: Bytes8!
}

type Library {
    id: ID!
    book: Book!
}
```

Two entities will be created: a `Book` entity, and a `Library` entity. As you can see, we add the `Book` entity as an attribute on the `Library` entity, thus conveying that we want a one-to-many or one-to-one relationship between `Library` and `Book`. This means that for a given `Library`, we may also fetch one or many `Book` entities.

# Foreign Keys

- The Fuel indexer service supports foreign key constraints using a combination of GraphQL schema and SQL backend (whether Postgres or SQLite).
- To demonstrate how the indexer uses GraphQL schema to resolve foreign key relationships, let's look at the following schema:

## Usage

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

### Breaking it down

- Given the above schema, two entities will be created: a `Book` entity, and a `Library` entity.
- We add the `Book` as a field on `Library`, thus conveying that we want a one-to-many or one-to-one relationship between `Library` and `Book`.

> Important:
>
> - At the moment, [due to some SQLite quirks](https://www.sqlite.org/omitted.html), the Fuel indexer SQLite support only offers foreign key _relationships_, not foreign key _constraints_. We are very much open to changing this in the future.

# Foreign Keys

- The Fuel indexer service supports forein key constraints and relationships using a combination of GraphQL schema and a database (whether Postgres or SQLite).
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
- As you can see, we add the `Book` entity as an attribute on the `Library` entity, thus conveying that we want a one-to-many or one-to-one relationship between `Library` and `Book`.
  - This means that for a given `Library`, we may also fetch one or many `Book` entities.


> Important:
>
> 1. At the moment, [due to some SQLite quirks](https://www.sqlite.org/omitted.html), the Fuel indexer SQLite support only offers foreign key _relationships_, not foreign key _constraints_. We are very much open to changing this in the future.
>
> 2. As mentioned in the [Conventions](./conventions.md) section, as of now, only `ID` types are supported for foreign key constraints. However, we do plan to support other types as well in the near future.

# Directives

> Per GraphQL: A directive is an identifier preceded by a @ character, optionally followed by a list of named arguments, which can appear after almost any form of syntax in the GraphQL query or schema languages.


## Fuel Indexer Directives

- As of now the Fuel indexer supports a single directive: `@indexed`.

### Usage

Using our `Library` and `Book` example from the previous [Foreign Keys](./foreign-keys.md) section -- given the following schema:

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
    name: Bytes8! @indexed
}

type Library {
    id: ID!
    book: Book!
}
```

A single `BTREE INDEX` constraint will be created on the `book` table's `name` column.


> Important: At the moment, index constraint support is limited to `BTREE` in Postgres with `ON DELETE`, and `ON UPDATE` actions not being supported. Note that `@indexed` directives are also available using SQLite. Finally, multi-column indices are _not_ supported at this time.

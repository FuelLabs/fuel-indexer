# Directives

> Per GraphQL: A directive is an identifier preceded by a @ character, optionally followed by a list of named arguments, which can appear after almost any form of syntax in the GraphQL query or schema languages.

- As of this writing list of supported Fuel GraphQL schema directives includes:

  - `@indexed`
  - `@unique`
  - `@join`

Using our `Library` and `Book` example from the previous [Foreign Keys](./foreign-keys.md) section -- given the following schema:

## `@indexed`

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

## `@unique`

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
    name: Bytes8! @unique
}

type Library {
    id: ID!
    book: Book!
}
```

A `UNIQUE` constraint will be created on the `book` table's `name` column.

> Important: When using explict or implicit foreign keys, it is required that the reference
column name in your foreign key relationship be unique. `ID` types are by default unique, but
all other types will have to be explicitly specified as being unique via the `@unique` directive.

## `@join`

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
    name: Bytes8! @unique
}

type Library {
    id: ID!
    book: Book! @join(on:name)
}
```

A foreign key constraint will be created on `library.book` that references `book.name`

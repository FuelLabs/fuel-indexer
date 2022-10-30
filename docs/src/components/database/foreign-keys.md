# Foreign Keys

- The Fuel indexer service supports foreign key constraints and relationships using a combination of GraphQL schema and a database (whether Postgres or SQLite).
- There are two types of uses for foreign keys - _implicit_ and _explicit_

> IMPORTANT:
>
> Implicit foreign keys do not require a `@join` directive. When using implicit foreign key references, merely add the referenced object as a field type (shown below). A lookup will automagically be done to add a foreign key constraint using this object's' `id` field.
>
> Note that implicit foreign key relationships _only_ use the `id` field on the referenced table. If you plan to use implicit foreign keys, the object being referenced _must_ have an `id` field.
>
> Explicit foreign keys on the other hand, _do_ require a `@join` directive. Explicit foreign key references work similarly to implicit foreign keys, however, when using explicit
foreign key references, you must add a `@join` directive after your object type. This `@join` directive includes the field in your foreign object that you would like to reference (shown below).

- To demonstrate how the indexer uses GraphQL schema to resolve foreign key relationships, let's look at the following schema:

## Usage

### Implicit foreign keys

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

#### Implicit foreign key breakdown

- Given the above schema, two entities will be created: a `Book` entity, and a `Library` entity.
- As you can see, we add the `Book` entity as an attribute on the `Library` entity, thus conveying that we want a one-to-many or one-to-one relationship between `Library` and `Book`.
  - This means that for a given `Library`, we may also fetch one or many `Book` entities.
  - This also means that the column `library.book` will be an integer type that references `book.id`

### Explicit foreign keys

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

#### Explicit foreign key breakdown

- For the most part, this works the same way as implicit foreign key usage
- However, as you can see, instead of implicitly using `book.id` as the reference column for our `Book` object, we're instead explicitly specifying that we want `book.name` to serve as our foreign key.
  - Also note that since we're using `book.name` in our foreign key constraint, that column is required to be unique -- via the `@unique` directive

> Important:
>
> 1. At the moment, [due to some SQLite quirks](https://www.sqlite.org/omitted.html), the Fuel indexer SQLite support only offers foreign key _relationships_, not foreign key _constraints_. We are very much open to changing this in the future.

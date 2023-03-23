# Queries

Once data has been persisted into your storage backend, you can retrieve it by querying the [GraphQL API server](./api-server.md). By default, the API server can be reached at `http://localhost:29987/api/graph/<namespace>/<identifier>`, where `<namespace>` and `<identifier>` are the values for the respective fields in your indexer's manifest. If you've changed the `GRAPHQL_API_HOST` or `GRAPHQL_API_PORT` values of your configuration, then you'll need to adjust the URL accordingly.

## Basic Query

A basic query has the following form:

```graphql
query {
    entity {
        field_1
        field_2
        ...
    }
    ...
}
```

The `entity` field corresponds to the name of an entity defined in your [schema](./schema.md) and the sub-fields are the fields defined on that entity type; entities and fields are stored in the database using the names defined in the schema, so make sure that your query uses those same names as well.

Let's refer back to the [block explorer](../../../examples/block-explorer.md) example for an illustration. After the block data has been indexed, we can retrieve information about the blocks by sending a query to the graph endpoint for our indexer.

```txt
curl -X POST http://localhost:29987/api/graph/fuel_examples/explorer_index \
   -H 'content-type: application/json' \
   -d '{"query": "query { block { id height timestamp }}", "params": "b"}' \
| json_pp
```

In the above snippet, you can see that we're requesting the ID, height, and timestamp for each block stored in the backend, and we're doing so by sending a `POST` request with a JSON payload. You can also see that the query is set as the value for the `query` key in the payload. If successful, the API server will return a response similar to the following:

```json
[
   {
      "height" : 1,
      "id" : "f169a30cfcbf1eebd97a07b19de98e4b38a4367b03d1819943be41744339d38a",
      "timestamp" : 1668710162
   },
   {
      "height" : 2,
      "id" : "a8c554758f78fe73054405d38099f5ad21a90c05206b5c6137424985c8fd10c7",
      "timestamp" : 1668710163
   },
   {
      "height" : 3,
      "id" : "850ab156ddd9ac9502768f779936710fd3d792e9ea79bc0e4082de96450b5174",
      "timestamp" : 1668710312
   },
   {
      "height" : 4,
      "id" : "19e19807c6988164b916a6877fe049d403d55a07324fa883cb7fa5cdb33438e2",
      "timestamp" : 1668710313
   },
   {
      "height" : 5,
      "id" : "363af43cfd2a6d8af166ee46c15276b24b130fc6a89ce7b3c8737d29d6d0e1bb",
      "timestamp" : 1668710314
   }
]
```

## Nested Query

The Fuel indexer supports [foreign keys](../database/foreign-keys.md) on entity types; thus, you can also ask for information about a referenced entity inside of your query. A nested query has the following general structure:

```graphql
query {
    entityA {
        field_A1
        field_A2
        referenced_entityB {
            field_B1
            field_B2
            ...
        }
        ...
    }
    ...
}
```

Essentially, it's the same as the basic query example with an added sub-block to request information about the reference entity. The response from the API server will be returned in the same general structure as the query. Let's look at another example to illustrate how it works in practice.

> Important: There is no limit to how deeply nested your entities and queries can be. However, every nested reference _**will**_ add computation and latency to your query as the information will have to be retrieved from different tables in your storage backend. Please exercise caution in your entity design and try to minimize nesting as much as possible.

We'll start with the following example schema:

```graphql
type QueryRoot {
    character: Character
    book: Book
    library: Library
    city: City
}

type City {
    id: ID!
    name: Charfield!
}

type Library {
    id: ID!
    name: Charfield!
    city: City!
}

type Book {
    id: ID!
    title: Charfield!
    library: Library!
}

type Character {
    id: ID!
    name: Charfield!
    book: Book!
}
```

This schema uses implicit foreign keys to reference other entities; for more information on implicit and explicit foreign keys, please refer to the [Foreign Keys](../database/foreign-keys.md) section of the book. In this contrived example, we're storing information about characters that are found in books which are stored in libraries that can be found in cities. This will be the query that we use to retrieve the aforementioned data:

```graphql
query {
    character {
        name
        book {
            title
            library {
                name
                city {
                    name
                }
            }
        }
    }
}
```

Let's assume that we've created an indexer for this data and indexed data has been stored in the database. Now we'll send a request to the API server in the same way that we did for the basic query example:

```txt
curl -X POST http://localhost:29987/api/graph/fuel_examples/nested_query_index \
   -H 'content-type: application/json' \
   -d '{"query": "query { character { name book { title library { name city { name } } } } }", "params": "b"}' \
| json_pp
```

And we receive the following response:

```json
[
  {
    "name": "Lil Ind X",
    "book": {
      "title": "Fuel Indexer",
      "library": {
        "name": "Fuel Labs Library",
        "city": {
          "name": "Fuel City"
        }
      }
    }
  }
]
```

# Pagination

The Fuel indexer currently supports offset-based pagination in order to allow users to selectively request parts of a set of results.

A paginated query can be made using three keywords:

- `order` - sort order (required)
- `first` - limit on number of results (required)
- `offset` - the amount of records to skip before returning results (optional)

```graphql
query {
  blocks: block(order: { height: asc }, first: 5) {
    hash
    height
    timestamp
  }
}
```

In this query, we're requesting details about the first five blocks in ascending order of block height. You can also see that we've added a `blocks` alias; this is completely optional and all it does is change the key for the list of results in the JSON response.

With this query, we receive the following response:

```json
{
  "data": {
    "blocks": [
      {
        "hash": "d77632f85669dd44737abf36b32f479ae518e07a9174c8571377ebb81563bb9a",
        "height": 1,
        "timestamp": 1678483351
      },
      {
        "hash": "7cb9542b624d88b3d66c6c9a1835f66fecba8892a87ffab9c17251c456ca5dcd",
        "height": 2,
        "timestamp": 1678483471
      },
      {
        "hash": "24f9611115f7ecb4a393751933a9f89329812cf08bdbe483c071b3401d06c8d6",
        "height": 3,
        "timestamp": 1678483591
      },
      {
        "hash": "616566afdc141ecd2b60fdc56aae4f3d04b3f6db9e65a3c21d0105a08cc1b349",
        "height": 4,
        "timestamp": 1678483711
      },
      {
        "hash": "1dca838d492f29b7a3afa7755ac8741c99db992da47673cd27be86f9b0620118",
        "height": 5,
        "timestamp": 1678483831
      }
    ],
    "page_info": {
      "has_next_page": true,
      "limit": 5,
      "offset": 0,
      "pages": 80,
      "total_count": 400
    }
  }
}
```

As you can see, we get the requested amount of blocks and the corresponding fields. However, there's also a `page_info` object included in the response. This object tells us if there's another page available to request along with information that we can use to construct our next response. To get the next page, we'll add an `offset` to our original query:

```graphql
query {
  blocks: block(
    order: { height: asc },
    first: 5,
    offset: 5
  ) {
    hash
    height
    timestamp
  }
}
```

The response contains the next five blocks _after_ our requested offset, and the `page_info` object lets us know what offset we used in the query.

```json
{
  "data": {
    "blocks": [
      {
        "hash": "c93ffc9178d526a836d707137de08b0f743fabce79ecec77c419bfb7e6be8863",
        "height": 6,
        "timestamp": 1678483951
      },
      {
        "hash": "4f0c81a42c86c718c0ae90ba838d6f1bdfc9a757cbf07c946fb3280b44257b46",
        "height": 7,
        "timestamp": 1678484071
      },
      {
        "hash": "659b486cc2c3bd1133df9245645648b6a09b35e16c7f71bb05449cea0e83611c",
        "height": 8,
        "timestamp": 1678484191
      },
      {
        "hash": "4bf61bd8f88b7fb40e842a6497d686bc2f63839ec3ca1eedb4e81a0935adaeb6",
        "height": 9,
        "timestamp": 1678484311
      },
      {
        "hash": "b090634788ddd0461cba4d0833a3f15b8e2f51e672fb1527fc8c78cd8f80dc1a",
        "height": 10,
        "timestamp": 1678484431
      }
    ],
    "page_info": {
      "has_next_page": true,
      "limit": 5,
      "offset": 5,
      "pages": 80,
      "total_count": 400
    }
  }
}
```

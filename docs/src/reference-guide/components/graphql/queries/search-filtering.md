# Search and Filtering

The Fuel indexer currently supports the following search and filtering operations:

- ID selection
- comparison
- set membership
- excluding null values

Additionally, you can combine these operations using the `and` or `or` keywords, and invert operations by using the `not` keyword.

> You should practice sensible database design when filtering records. Apply database indicies to the underlying columns in order to make search operations more efficient; however, be advised that an overuse of database indicies will lead to degraded performance.

## ID Selection

You can query for a particular instance of an object by passing an ID value to with the `id` key. For example, let's say that you are storing blocks and you want details about a certain block, you would pass the corresponding ID:

```graphql
query {
  block(id: 4121419699470229811) {
    id
    hash
    height
    producer
  }
}
```

```json
{
  "data": [
    {
      "hash": "aff5eb785f2d24ae62858fa673296e957abea518858e2f08bb47df2dbb9c8ca1",
      "height": 8209,
      "id": 4121419699470229811,
      "producer": "f65d6448a273b531ee942c133bb91a6f904c7d7f3104cdaf6b9f7f50d3518871"
    }
  ]
}
```

> Note: Remember that IDs currently must be of type `u64`, and as such, the ID selection operation will only allow for a `u64` value. We're open to changing this in the future.

## Excluding Null Values

You can store null values in your records if the corresponding entity fields are configured to allow for it. You can exclude records that contain null values in a particular column or set of coulmns by using the `has` operator inside of a `filter` object.

```graphql
query {
  block(filter: { has: [producer] } ) {
    id
    hash
    height
    producer
  }
}
```

```json
{
  "data": [
    {
      "hash": "d0d663e0bf499aca19d3ecb9b0b291234dc3769d2b46512016eca7244ca0ef22",
      "height": 411,
      "id": 3775485677453992400,
      "producer": "f65d6448a273b531ee942c133bb91a6f904c7d7f3104cdaf6b9f7f50d3518871"
    },
    {
      "hash": "7ff79bf3793143c557225f37b7e7d8a2b9d2e544b839d62cc367b8c5b079d478",
      "height": 412,
      "id": 3919088689958184000,
      "producer": "f65d6448a273b531ee942c133bb91a6f904c7d7f3104cdaf6b9f7f50d3518871"
    }
  ]
}
```

## Set Membership

Additionally, you can exclude records in which a particular column's value does not contain any elements in a given set by using the `in` operator as part of a `filter` object.

```graphql
query {
  block(filter: { height: { in: [1, 2, 3] } } ) {
    id
    hash
    height
  }
}
```

```json
{
  "data": [
    {
      "hash": "d77632f85669dd44737abf36b32f479ae518e07a9174c8571377ebb81563bb9a",
      "height": 1,
      "id": 3618468784755926500
    },
    {
      "hash": "7cb9542b624d88b3d66c6c9a1835f66fecba8892a87ffab9c17251c456ca5dcd",
      "height": 2,
      "id": 4122538829619016000
    },
    {
      "hash": "24f9611115f7ecb4a393751933a9f89329812cf08bdbe483c071b3401d06c8d6",
      "height": 3,
      "id": 3762867646901937000
    }
  ]
}
```

## Comparison

Finally, you can filter records by comparing the values of certain fields to a particular value of your choosing by using one of the comparison operators inside of a `filter` object.

### Less, Greater, and Equals

You can do simple value comparisons using any of the following operators:

| | |
|------|----------|
| equals | `equals` |
| greater than | `gt` |
| greater than or equal to | `gte` |
| less than | `lt` |
| less than or equal to | `lte`   |

Here's an example:

```graphql
query {
  block(filter: { height: { lte: 5 } } ) {
    id
    hash
    height
    producer
  }
}
```

```json
{
  "data": [
    {
      "hash": "d77632f85669dd44737abf36b32f479ae518e07a9174c8571377ebb81563bb9a",
      "height": 1,
      "id": 3618468784755926500,
      "producer": "f65d6448a273b531ee942c133bb91a6f904c7d7f3104cdaf6b9f7f50d3518871"
    },
    {
      "hash": "7cb9542b624d88b3d66c6c9a1835f66fecba8892a87ffab9c17251c456ca5dcd",
      "height": 2,
      "id": 4122538829619016000,
      "producer": "f65d6448a273b531ee942c133bb91a6f904c7d7f3104cdaf6b9f7f50d3518871"
    },
    {
      "hash": "24f9611115f7ecb4a393751933a9f89329812cf08bdbe483c071b3401d06c8d6",
      "height": 3,
      "id": 3762867646901937000,
      "producer": "f65d6448a273b531ee942c133bb91a6f904c7d7f3104cdaf6b9f7f50d3518871"
    },
    {
      "hash": "616566afdc141ecd2b60fdc56aae4f3d04b3f6db9e65a3c21d0105a08cc1b349",
      "height": 4,
      "id": 3833467323683451000,
      "producer": "f65d6448a273b531ee942c133bb91a6f904c7d7f3104cdaf6b9f7f50d3518871"
    },
    {
      "hash": "1dca838d492f29b7a3afa7755ac8741c99db992da47673cd27be86f9b0620118",
      "height": 5,
      "id": 3991987200693004000,
      "producer": "f65d6448a273b531ee942c133bb91a6f904c7d7f3104cdaf6b9f7f50d3518871"
    }
  ]
}
```

### Between

You can also filter for records that are contained in the range of two values by using the `between` operator. To do so, you'd set the lower bound using the `min` keyword and the upper bound by using `max`.

```graphql
query {
  block(filter: { height: { between: { min: 101, max: 103 } } } ) {
    id
    hash
    height
    producer
  }
}
```

```json
{
  "data": [
    {
      "hash": "3b85fbed2d933d0334d54776612a5af72a513e875d06fa9152f6d41d0e50e417",
      "height": 101,
      "id": 3763145849079675000,
      "producer": "f65d6448a273b531ee942c133bb91a6f904c7d7f3104cdaf6b9f7f50d3518871"
    },
    {
      "hash": "deea78034c2f0fcd7ef2d2d2d203d19fcd63f1b1846fac089c51c2aa7b5c8149",
      "height": 102,
      "id": 7365137137742930000,
      "producer": "f65d6448a273b531ee942c133bb91a6f904c7d7f3104cdaf6b9f7f50d3518871"
    },
    {
      "hash": "a405d5688fdf41817868361217a09812349cc6fe0fe2bf9329fcd23e338e9444",
      "height": 103,
      "id": 7292000934927820000,
      "producer": "f65d6448a273b531ee942c133bb91a6f904c7d7f3104cdaf6b9f7f50d3518871"
    }
  ]
}
```

## Logical Operators

As previously stated, you can combine or invert operations to filter for your desired results even further.

### And/Or

Let's look at an example query in which we combine two filters together.

```graphql
query {
  block(filter: { 
    producer: { equals: "f65d6448a273b531ee942c133bb91a6f904c7d7f3104cdaf6b9f7f50d3518871" },
    and: { height: { lt: 4 } }
  } ) {
    id
    hash
    height
    producer
  }
}
```

```json
{
  "data": [
    {
      "hash": "d77632f85669dd44737abf36b32f479ae518e07a9174c8571377ebb81563bb9a",
      "height": 1,
      "id": 3618468784755926500,
      "producer": "f65d6448a273b531ee942c133bb91a6f904c7d7f3104cdaf6b9f7f50d3518871"
    },
    {
      "hash": "7cb9542b624d88b3d66c6c9a1835f66fecba8892a87ffab9c17251c456ca5dcd",
      "height": 2,
      "id": 4122538829619016000,
      "producer": "f65d6448a273b531ee942c133bb91a6f904c7d7f3104cdaf6b9f7f50d3518871"
    },
    {
      "hash": "24f9611115f7ecb4a393751933a9f89329812cf08bdbe483c071b3401d06c8d6",
      "height": 3,
      "id": 3762867646901937000,
      "producer": "f65d6448a273b531ee942c133bb91a6f904c7d7f3104cdaf6b9f7f50d3518871"
    }
  ]
}
```

### Not

You can also use the `not` operator in order to invert the operation of a particular filter. For example, the following query returns contracts that we've seen on the network.

```graphql
query {
  contract {
    id
  }
}
```

```json
{
  "data": [
    {
      "id": "1072ca8fcab43048a5b31c1ea204748c2cb5acca6b90f3b1a02ef7a2d92386d9"
    },
    {
      "id": "9b8b258e0d64b9e8a022e3f38a751ad5a1b36e4dfdcc25a6fb8308e044250b8c"
    },
    {
      "id": "0000000000000000000000000000000000000000000000000000000000000000"
    },
    {
      "id": "8fe8ce43603c1a48274aac7532da56707901d9606a2b05de801993f48ea6bfe7"
    }
  ]
}
```

Let's ignore the base asset contract by inverting the `in` operator:

```graphql
query {
  contract(filter: {not: { id: { equals: "0000000000000000000000000000000000000000000000000000000000000000"}}}) {
    id
  }
}
```

```json
{
  "data": [
    {
      "id": "1072ca8fcab43048a5b31c1ea204748c2cb5acca6b90f3b1a02ef7a2d92386d9"
    },
    {
      "id": "9b8b258e0d64b9e8a022e3f38a751ad5a1b36e4dfdcc25a6fb8308e044250b8c"
    },
    {
      "id": "8fe8ce43603c1a48274aac7532da56707901d9606a2b05de801993f48ea6bfe7"
    }
  ]
}
```

# Search and Filtering

The Fuel indexer currently supports the following search and filtering operations:

- ID selection
- comparison
- set membership
- excluding null values

Additionally, you can combine these operations using the `and` or `or` keywords, and invert operations by using the `not` keyword.

To illustrate these operations, we'll use a `filter_example` entity that would typically return the following results with a regular, unfiltered query:

```graphql
query {
  filter_example {
    id
    foo
    possibly_null_bar
    baz
  }
}
```

```json
{
  "data": [
    {
      "baz": 1,
      "foo": "beep",
      "id": 1,
      "possibly_null_bar": 123
    },
    {
      "baz": 5,
      "foo": "boop",
      "id": 2,
      "possibly_null_bar": null
    },
    {
      "baz": 1000,
      "foo": "blorp",
      "id": 3,
      "possibly_null_bar": 456
    }
  ]
}
```

> You should practice sensible database design when filtering records. Apply database indicies to the underlying columns in order to make search operations more efficient; however, be advised that an overuse of database indicies will lead to degraded performance.

## ID Selection

You can query for a particular instance of an object by passing an ID value to with the `id` key. For example, let's say that you are storing blocks and you want details about a certain block, you would pass the corresponding ID:

```graphql
query {
  filter_example(id: 1) {
    id
    foo
    possibly_null_bar
    baz
  }
}
```

```json
{
  "data": [
    {
      "baz": 1,
      "foo": "beep",
      "id": 1,
      "possibly_null_bar": 123
    }
  ]
}
```

> Note: Remember that IDs currently must be of type `u64`, and as such, the ID selection operation will only allow for a `u64` value. We're open to changing this in the future.

## Excluding Null Values

You can store null values in your records if the corresponding entity fields are configured to allow for it. You can exclude records that contain null values in a particular column or set of coulmns by using the `has` operator inside of a `filter` object.

If you wanted to filter out any records with a null value in `possibly_null_bar`, you would use the following query:

```graphql
query {
  filter_example(filter: { has: [possibly_null_bar] } ) {
    id
    foo
    possibly_null_bar
    baz
  }
}
```

```json
{
  "data": [
    {
      "baz": 1,
      "foo": "beep",
      "id": 1,
      "possibly_null_bar": 123
    },
    {
      "baz": 1000,
      "foo": "blorp",
      "id": 3,
      "possibly_null_bar": 456
    }
  ]
}
```

## Set Membership

Additionally, you can exclude records in which a particular column's value does not contain any elements in a given set by using the `in` operator as part of a `filter` object.

```graphql
query {
  filter_example(filter: { foo: { in: ["beep", "boop"] } } ) {
    id
    foo
    possibly_null_bar
    baz
  }
}
```

```json
{
  "data": [
    {
      "baz": 1,
      "foo": "beep",
      "id": 1,
      "possibly_null_bar": 123
    },
    {
      "baz": 5,
      "foo": "boop",
      "id": 2,
      "possibly_null_bar": null
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
  filter_example(filter: { baz: { gte: 5 } } ) {
    id
    foo
    possibly_null_bar
    baz
  }
}
```

```json
{
  "data": [
    {
      "baz": 5,
      "foo": "boop",
      "id": 2,
      "possibly_null_bar": null
    },
    {
      "baz": 1000,
      "foo": "blorp",
      "id": 3,
      "possibly_null_bar": 456
    }
  ]
}
```

### Between

You can also filter for records that are contained in the range of two values by using the `between` operator. To do so, you'd set the lower bound using the `min` keyword and the upper bound by using `max`.

```graphql
query {
  filter_example(filter: { baz: { between: { min: 0, max: 10 } } } ) {
    id
    foo
    possibly_null_bar
    baz
  }
}
```

```json
{
  "data": [
    {
      "baz": 1,
      "foo": "beep",
      "id": 1,
      "possibly_null_bar": 123
    },
    {
      "baz": 5,
      "foo": "boop",
      "id": 2,
      "possibly_null_bar": null
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
  filter_example(filter: { 
    has: [possibly_null_bar], 
    and: { 
        baz: { equals: 1 } 
    } 
  }) {
    id
    foo
    possibly_null_bar
    baz
  }
}
```

With this query, we're looking for records in which `possibly_null_bar` is _not_ null **and** the value for `baz` is equal to `1`.

```json
{
  "data": [
    {
      "baz": 1,
      "foo": "beep",
      "id": 1,
      "possibly_null_bar": 123
    }
  ]
}
```

### Not

You can also use the `not` operator in order to invert the operation of a particular filter. For example, in the following query, we're looking for any records in which the value for `foo` does not match "beep" or "boop".

```graphql
query {
  filter_example(filter: { not: { foo: { in: ["beep", "boop"] } } } ) {
    id
    foo
    possibly_null_bar
    baz
  }
}
```

```json
{
  "data": [
    {
      "baz": 1000,
      "foo": "blorp",
      "id": 3,
      "possibly_null_bar": 456
    }
  ]
}
```

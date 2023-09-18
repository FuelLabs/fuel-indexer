# A Full Example

Finally, let's combine nested entities, filtering, and pagination into one complete example.

Sticking with the same block explorer example, let's say that we are looking for a particular transaction and its containing block, but we don't remember either of the hashes. All we know is that the total value of the transaction is greater than zero, it was sometime after the start of the `beta-4` testnet, and it was included as part of the first fifty blocks. Additionally, we don't want to parse through all the results at once, so we only want to look at two records at a time. Finally, we think that it may have been on the more recent side, so we want to check them in reverse chronological order.

Putting all of that together, we get the following query:

```graphql
query {
  transactions: tx(
    order: { timestamp: desc },
    filter: { value: { gt: 0 } },
    first: 2,
    offset: 0
  ) {
    id
    hash
    timestamp
    value
    block (
      filter: { 
        height: { between: { min: 0, max: 50 } }, 
        and: { 
          timestamp: { gt: 1678410000 } 
        }
      } 
    ) {
      id
      hash
      height
      timestamp
    }
  }
}
```

The Fuel indexer's GraphQL API allows you to add filters on multiple entity fields and even nested entities! In the query above, we're asking for the two most recent transactions with a value greater than zero. Also, we're applying two filters to the nested `block` entity by using the `and` operator in order to help us narrow down the set of results.

The response returns the results in the expected format and includes additional information that informs us about how many total results satisy the criteria.

```json
{
  "data": {
    "page_info": {
      "has_next_page": true,
      "limit": 2,
      "offset": 0,
      "pages": 2,
      "total_count": 4
    },
    "transactions": [
      {
        "block": {
          "hash": "f40297895086e66c0947c213dd29e90f596b860d10316ab806064608dd2580cd",
          "height": 45,
          "id": 7306026486395921000,
          "timestamp": 1678486898
        },
        "hash": "85acfa181ebfa3b48c10d3181217918dd377b875d07dabc72d6d1081e4c52713",
        "id": 3919319574514776000,
        "timestamp": 1678486898,
        "value": 10000000000
      },
      {
        "block": {
          "hash": "e3e0860a358c0d044669748cffff82b4b0073baaca53a128ddc8ce3757ae3988",
          "height": 41,
          "id": 7018409465212200000,
          "timestamp": 1678486633
        },
        "hash": "42f3fd7ffa073975a0eca993044a867d8c87a8d39f5a88032a3b9aba213f6102",
        "id": 7364622549171910000,
        "timestamp": 1678486633,
        "value": 10000000000
      }
    ]
  }
}
```

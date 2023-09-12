# Object types

Object types are the most commonly used type in indexer GraphQL schema. Each object type marked with an `@entity` directive will be converted into a SQL table. 

```graphql
type Account @entity {
    id: ID!
    address: Address!
    balance: UInt8!
}
```

This `Account` object type from the GraphQL schema, might be used in an indexer module like so:

```rust, ignore
extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "indexer.manifest.yaml")]
mod indexer_mod {
    fn handle_event(event: Event) {
        let address = Address::default();
        let balance = 0;
        let account = Account::new(address, balance);
        account.save();
    }
}
```
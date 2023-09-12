# Enum Types

Enum types are simply implemented as String types.

```graphql
enum SignatureLabel {
    Multi
    Single
}
```

> Enum types in relation to Fuel indexer's implementation are just `String` types used primarily to label object types. There is no other way that `enum` types should be used at this time.

This `SignatureLabel` object type from the GraphQL schema, might be used in an indexer module like so:

```rust, ignore
extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "indexer.manifest.yaml")]
mod indexer_mod {
    fn handle_event(event: Event) {
        let label = SignatureLabel::Multi;
        assert_eq!(label.to_string(), "SignatureLabel::Multi".to_string());
    }
}
```
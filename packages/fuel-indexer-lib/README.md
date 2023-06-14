# fuel-indexer-lib

A collection of utilities used by the various `fuel-indexer-*` crates.

```rust
use fuel_indexer_lib::utils::trim_opt_env_key;

fn main() {
    let a = "${ENV_VAR_KEY}";
    assert_eq!(trim_opt_env_key(a), "ENV_VAR_KEY");
}
```

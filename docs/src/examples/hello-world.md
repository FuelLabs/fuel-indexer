# Hello World

Below is a simple "Hello World" Sway contract that we want to index. This contract has a function called `new_greeting` that logs a `Greeting` and a `Person`.

```rust, ignore
{{#include ../../../examples/hello-world/contracts/greeting/src/main.sw}}
```

We can define our schema like this in the schema file:

```graphql
{{#include ../../../examples/hello-world/hello-index/schema/hello_index.schema.graphql}}
```

Now that our schema is defined, here is how we can implement the WASM module in our `lib.rs` file:

```rust,ignore
{{#include ../../../examples/hello-world/hello-index/src/lib.rs}}
```

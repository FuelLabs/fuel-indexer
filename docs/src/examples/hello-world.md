# Hello World

A "Hello World" type of program for the Fuel Indexer service.

```rust
//! A rudimentary block explorer implementation demonstrating how blocks, transactions,
//! contracts, and accounts can be persisted into the database.
//!
//! Build this example's WASM module using the following command. Note that a
//! wasm32-unknown-unknown target will be required.
//!
//! ```bash
//! cargo build -p hello-index --release --target wasm32-unknown-unknown
//! ```
//!
//! Start a local test Fuel node
//!
//! ```bash
//! cargo run --bin fuel-node
//! ```
//!
//! With your database backend set up, now start your fuel-indexer binary using the
//! assets from this example:
//!
//! ```bash
//! cargo run --bin fuel-indexer -- --manifest examples/hello-world/hello-index.manifest.yaml
//! ```
//!
//! Now trigger an event.
//!
//! ```bash
//! cargo run --bin hello-bin
//! ```

extern crate alloc;
use fuel_indexer_macros::indexer;
use fuel_indexer_plugin::{types::Bytes32, utils::sha256_digest};

// A utility function used to convert an arbitrarily sized string into Bytes32
// using the first 32 bytes of the String. This might be provided by a standard-ish
// library in the future.
fn bytes32(data: &str) -> Bytes32 {
    let data = sha256_digest(&data);
    let mut buff = [0u8; 32];
    buff.copy_from_slice(&data.as_bytes()[..32]);
    Bytes32::from(buff)
}

// A utility function used to convert an arbitrarily sized string into u64
// using the first 8 bytes of the String. This might be provided by a standard-ish
// library in the future.
fn u64_id(data: &str) -> u64 {
    let mut buff = [0u8; 8];
    buff.copy_from_slice(&data.as_bytes()[..8]);
    u64::from_le_bytes(buff)
}

#[indexer(manifest = "examples/hello-world/hello-index.manifest.yaml")]
mod hello_world_index {
    fn index_logged_greeting(event: Greeting, block: BlockData) {
        // Since all events require a u64 ID field, let's derive an ID using the
        // name of the person in the Greeting
        let greeter_id = u64_id(&event.person.name.to_string());

        // Here we 'get or create' a Salutation based on the ID of the event
        // emiited in the LogData receipt of our smart contract
        let greeting = match Salutation::load(event.id) {
            Some(mut g) => {
                // If we found an event, let's use block height as a proxy for time
                g.last_seen = block.height;
                g
            }
            None => {
                // If we did not already have this Saluation stored in the database. Here we
                // show how you can use the Charfield type to store strings with length <= 255
                let message =
                    format!("{} 👋, my name is {}", &event.greeting, &event.person.name);

                Salutation {
                    id: event.id,
                    message_hash: bytes32(&message),
                    message,
                    greeter: greeter_id,
                    first_seen: block.height,
                    last_seen: block.height,
                }
            }
        };

        // Here we do the same with Greeter that we did for Saluation -- if we have an event
        // already saved in the database, load it and update it. If we do not have this Greeter
        // in the database then create one
        let greeter = match Greeter::load(greeter_id) {
            Some(mut g) => {
                g.last_seen = block.height;
                g
            }
            None => Greeter {
                id: greeter_id,
                first_seen: block.height,
                name: event.person.name.to_string(),
                last_seen: block.height,
            },
        };

        // Both entity saves will occur in the same transaction
        greeting.save();
        greeter.save();
    }
}
```

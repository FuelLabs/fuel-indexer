//! A "Hello World" type of program for the Fuel Indexer service.
//!
//! Build this example's WASM module using the following command. Note that a
//! wasm32-unknown-unknown target will be required.
//!
//! ```bash
//! cargo build -p hello-indexer --release --target wasm32-unknown-unknown
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
//! cargo run --bin fuel-indexer -- --manifest examples/hello-world/hello-indexer/hello_indexer.manifest.yaml
//! ```
//!
//! Now trigger an event.
//!
//! ```bash
//! cargo run --bin hello-bin
//! ```

extern crate alloc;
use fuel_indexer_macros::indexer;
use fuel_indexer_plugin::prelude::*;

#[indexer(manifest = "examples/hello-world/hello-indexer/hello_indexer.manifest.yaml")]
mod hello_world_indexer {

    fn index_logged_greeting(event: Greeting, block: BlockData) {
        // Since all events require a u64 ID field, let's derive an ID using the
        // name of the person in the Greeting
        let greeter_name = trim_sized_ascii_string(&event.person.name);
        let greeting = trim_sized_ascii_string(&event.greeting);
        let greeter_id = first8_bytes_to_u64(&greeter_name);

        // Here we 'get or create' a Salutation based on the ID of the event
        // emitted in the LogData receipt of our smart contract
        let salutation = match Salutation::load(event.id) {
            Some(mut g) => {
                // If we found an event, let's use block height as a proxy for time
                g.last_seen = block.height;
                g
            }
            None => {
                // If we did not already have this Saluation stored in the database. Here we
                // show how you can use the Charfield type to store strings with length <= 255
                let message = format!("{} 👋, my name is {}", &greeting, &greeter_name);

                Salutation {
                    id: event.id,
                    message_hash: first32_bytes_to_bytes32(&message),
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
                name: greeter_name,
                last_seen: block.height,

                // Here we show an example of an arbtrarily sized Blob type. These Blob types
                // support data up to 10485760 bytes in length
                visits: vec![1u8, 2, 3, 4, 5, 6, 7, 8],
            },
        };

        // Both entity saves will occur in the same transaction
        salutation.save();
        greeter.save();
    }
}

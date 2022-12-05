//! A "Hello World" type of program for the Fuel Indexer service that uses native execution.
//!
//! Build this example's binary using the following command.
//!
//! ```bash
//! cargo run --bin hello-index-native -- --manifest examples/hello-world-native/hello_index_native.manifest.yaml
//! ```
//!
//! Start a local test Fuel node
//!
//! ```bash
//! cargo run --bin fuel-node
//! ```
//!
//! Now trigger an event.
//!
//! ```bash
//! cargo run --bin hello-bin
//! ```
extern crate alloc;
use fuel_indexer::prelude::*;
use fuel_indexer_macros::indexer;
use fuel_indexer_plugin::prelude::*;

#[indexer(manifest = "examples/hello-world-native/hello_index_native.manifest.yaml")]
mod hello_world_native {
    async fn index_logged_greeting(event: Greeting, block: BlockData) {
        // Since all events require a u64 ID field, let's derive an ID using the
        // name of the person in the Greeting
        let greeter_id = first8_bytes_to_u64(&event.person.name.to_string());

        // Here we 'get or create' a Salutation based on the ID of the event
        // emitted in the LogData receipt of our smart contract
        let greeting = match Salutation::load(event.id).await {
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
        let greeter = match Greeter::load(greeter_id).await {
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
        greeting.save().await;
        greeter.save().await;
    }
}

//! A "Hello World" type of program for the Fuel Indexer service that uses native execution.
//!
//! Build this example's binary using the following command.
//!
//! Start a local test Fuel node
//!
//! ```bash
//! cargo run -p hello-world-node --bin hello-world-node
//! ```
//!
//! With your database backend set up, now start your fuel-indexer binary using the
//! assets from this example:
//!
//! ```bash
//! cargo run -p hello_indexer_native --bin hello_indexer_native -- --manifest examples/hello-world-native/hello-indexer-native/hello_indexer_native.manifest.yaml
//! ```
//!
//! Now trigger an event.
//!
//! ```bash
//! cargo run -p hello-world-data --bin hello-world-data
//! ```
use fuel_indexer_utils::prelude::*;

#[indexer(
    manifest = "examples/hello-world-native/hello-indexer-native/hello_indexer_native.manifest.yaml"
)]
mod hello_world_native {

    async fn index_logged_greeting(event: Greeting, block: BlockData) {
        let greeting = event.greeting.to_right_trimmed_str().to_string();
        let name = event.person.name.to_right_trimmed_str().to_string();
        let height = block.height;
        let data = vec![1u8, 2, 3, 4, 5, 6, 7, 8].into();
        let greeter = Greeter::new(name.clone(), height, height, data)
            .get_or_create()
            .await;

        let message = format!("{greeting} 👋, my name is {name}");
        let message_hash = bytes32(&message);

        let salutation =
            Salutation::new(message_hash, message, greeter.id.clone(), height, height)
                .get_or_create()
                .await;

        greeter.save().await;
        salutation.save().await;
    }
}

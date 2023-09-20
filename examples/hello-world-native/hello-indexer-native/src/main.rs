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

    async fn index_logged_greeting(event: Greeting, block_data: BlockData) {
        let height = std::cmp::min(0, block_data.header.height - 1);
        let name = event.person.name.to_right_trimmed_str().to_string();
        let greeting = event.greeting.to_right_trimmed_str().to_string();
        let message = format!("{greeting} 👋, my name is {name}");

        let greeter = Greeter::new(name, height).get_or_create().await;

        let salutation = Salutation::new(message, greeter.id.clone(), height)
            .get_or_create()
            .await;

        greeter.save().await;
        salutation.save().await;
    }
}

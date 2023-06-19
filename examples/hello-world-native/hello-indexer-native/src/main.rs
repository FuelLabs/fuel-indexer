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
extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(
    manifest = "examples/hello-world-native/hello-indexer-native/hello_indexer_native.manifest.yaml"
)]
mod hello_world_native {

    async fn index_logged_greeting(event: Greeting, block: BlockData) {
        // We're using the `::new()` method to create a Greeter, which automatically
        // generates an ID for the entity. Then, we use `::get_or_create()` to
        // load the corresponding record from the database, if present.
        let greeter = Greeter::new(
            trim_sized_ascii_string(&event.person.name),
            block.height,
            block.height,
            vec![1u8, 2, 3, 4, 5, 6, 7, 8].into(),
        )
        .get_or_create()
        .await;

        // Here we show how you can use the Charfield type to store strings with
        // length <= 255. The fuel-indexer-utils package contains a number of helpful
        // functions for byte conversion, string manipulation, etc.
        let message = format!(
            "{} 👋, my name is {}",
            trim_sized_ascii_string(&event.greeting),
            trim_sized_ascii_string(&event.person.name)
        );
        let message_hash = first32_bytes_to_bytes32(&message);

        let salutation = Salutation::new(
            message_hash,
            message,
            greeter.id,
            block.height,
            block.height,
        )
        .get_or_create()
        .await;

        // Finally, we save the entities to the database.
        greeter.save().await;
        salutation.save().await;
    }
}

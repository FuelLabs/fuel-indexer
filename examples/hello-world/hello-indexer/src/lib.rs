//! A "Hello World" type of program for the Fuel Indexer service.
//!
//! Build this example's WASM module using the following command. Note that a
//! wasm32-unknown-unknown target will be required.
//!
//! ```bash
//! cargo build -p hello-indexer --release --target wasm32-unknown-unknown
//! ```
//!
//! Start a local test Fuel node.
//!
//! ```bash
//! cargo run -p hello-world-node --bin hello-world-node
//! ```
//!
//! With your database backend set up, now start your fuel-indexer binary using the
//! assets from this example:
//!
//! ```bash
//! cargo run --bin fuel-indexer -- run --manifest examples/hello-world/hello-indexer/hello_indexer.manifest.yaml --run-migrations
//! ```
//!
//! Now trigger an event.
//!
//! ```bash
//! cargo run -p hello-world-data --bin hello-world-data
//! ```

extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "examples/hello-world/hello-indexer/hello_indexer.manifest.yaml")]
mod hello_world_indexer {

    fn index_logged_greeting(event: Greeting, block: BlockData) {
        let greeting = event.greeting.to_right_trimmed_str().to_string();
        let name = event.person.name.to_right_trimmed_str().to_string();
        let height = block.height;
        let data = vec![1u8, 2, 3, 4, 5, 6, 7, 8].into();
        let greeter = Greeter::new(name.clone(), height, height, data).get_or_create();

        let message = format!("{greeting} 👋, my name is {name}");
        let message_hash = bytes32(&message);

        let salutation =
            Salutation::new(message_hash, message, greeter.id.clone(), height, height)
                .get_or_create();

        greeter.save();
        salutation.save();
    }
}

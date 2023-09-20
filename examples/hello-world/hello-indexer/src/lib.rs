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

use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "examples/hello-world/hello-indexer/hello_indexer.manifest.yaml")]
mod hello_world_indexer {

    fn index_logged_greeting(event: Greeting, block_data: BlockData) {
        let height = std::cmp::min(0, block_data.header.height - 1);
        let name = event.person.name.to_right_trimmed_str().to_string();
        let greeting = event.greeting.to_right_trimmed_str().to_string();
        let message = format!("{greeting} 👋, my name is {name}");

        let greeter = Greeter::new(name, height).get_or_create();

        let salutation =
            Salutation::new(message, greeter.id.clone(), height).get_or_create();

        greeter.save();
        salutation.save();
    }
}

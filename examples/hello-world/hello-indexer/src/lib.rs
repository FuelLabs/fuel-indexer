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
//! cargo run -p fuel-node --bin fuel-node
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
//! cargo run -p hello-bin --bin hello-bin
//! ```

extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "examples/hello-world/hello-indexer/hello_indexer.manifest.yaml")]
mod hello_world_indexer {

    fn index_logged_greeting(event: Greeting, block: BlockData) {
        // Since all events require a u64 ID field, let's derive an ID using the
        // name of the person in the Greeting
        let greeter_name = trim_sized_ascii_string(&event.person.name);
        let greeting = trim_sized_ascii_string(&event.greeting);
        let greeter_id = first8_bytes_to_u64(&greeter_name);

        // If we have a greeter already saved in the database, load it and update it.
        // If we do not have a Greeter with this ID in the database, then create one.
        let greeter = match Greeter::load(greeter_id) {
            Some(mut g) => {
                g.last_seen = block.height;
                g
            }
            None => Greeter {
                id: greeter_id,
                first_seen: block.height,
                name: greeter_name.clone(),
                last_seen: block.height,

                // Here we show an example of an arbtrarily sized Blob type. These Blob types
                // support data up to 10485760 bytes in length
                visits: vec![1u8, 2, 3, 4, 5, 6, 7, 8].into(),
            },
        };
        greeter.save();

        // If we did not already have this Saluation stored in the database. Here we
        // show how you can use the Charfield type to store strings with length <= 255
        let message = format!("{} 👋, my name is {}", &greeting, &greeter_name);
        let message_hash = first32_bytes_to_bytes32(&message);

        // Here we do the same thing for Salutation based on the ID of the event
        // emitted in the LogData receipt of our smart contract.
        let salutation = match Salutation::load(event.id) {
            Some(mut g) => {
                // If we found an event, let's use block height as a proxy for time
                g.last_seen = block.height;
                g
            }
            None => Salutation {
                id: event.id,
                message_hash,
                message: message.clone(),
                greeter: greeter_id,
                first_seen: block.height,
                last_seen: block.height,
            },
        };
        salutation.save();

        // You can also use the `get_or_create()` method to load a record from the database or create
        // an entity, if there is no corresponding record for the given ID.
        let (greeter_by_get_create, greeter_created): (Greeter, bool) =
            Greeter::get_or_create(
                greeter_id,
                Greeter {
                    id: greeter_id,
                    first_seen: block.height,
                    name: greeter_name.clone(),
                    last_seen: block.height,
                    visits: vec![1u8, 2, 3, 4, 5, 6, 7, 8].into(),
                },
            );

        if greeter_created {
            Logger::info("Loaded greeter from the DB");
        }
        greeter_by_get_create.save();

        let (salutation_by_get_create, salutation_created): (Salutation, bool) =
            Salutation::get_or_create(
                event.id,
                Salutation {
                    id: event.id,
                    message_hash,
                    message: message.clone(),
                    greeter: greeter_id,
                    first_seen: block.height,
                    last_seen: block.height,
                },
            );
        if salutation_created {
            Logger::info("Loaded salutation from the DB");
        }
        salutation_by_get_create.save();

        // Finally, you can also use the `new()` method to instantiate an entity with the ID field
        // derived from a SHA-256 hash of the content of the other entity fields.
        let greeter_by_new = Greeter::new(
            trim_sized_ascii_string(&event.person.name),
            block.height,
            block.height,
            vec![1u8, 2, 3, 4, 5, 6, 7, 8].into(),
        );
        greeter_by_new.save();

        let salutation_by_new = Salutation::new(
            message_hash,
            message.clone(),
            greeter.id,
            block.height,
            block.height,
        );
        salutation_by_new.save();
    }
}

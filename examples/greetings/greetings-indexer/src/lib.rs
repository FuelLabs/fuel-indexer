extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(
    manifest = "examples/greetings/greetings-indexer/greetings_indexer.manifest.yaml"
)]
mod greetings_indexer {

    fn greetings_indexer_handler(event: Greeting, block_data: BlockData) {
        info!("Handling new Greeting event.");
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

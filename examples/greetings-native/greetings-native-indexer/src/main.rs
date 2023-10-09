extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(
    manifest = "examples/greetings-native/greetings-native-indexer/greetings_native_indexer.manifest.yaml"
)]
mod greetings_native_indexer {

    async fn greetings_indexer_handler(event: Greeting, block_data: BlockData) {
        info!("Handling new Greeting event.");
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

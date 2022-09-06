extern crate alloc;
use fuel_indexer_macros::indexer;

#[indexer(
    abi = "composable-indexer-lib/contracts/ping/out/debug/ping-abi.json",
    namespace = "composability_test",
    schema = "composable-indexer-lib/schema/schema.graphql"
)]
mod composability_test {
    fn function_one(ping: Ping) {
        Logger::info("function_one handling a Pong event.");

        let mut bytes: [u8; 32] = [0u8; 32];

        bytes.copy_from_slice(&ping.message.as_bytes()[..32]);

        let entity = Message {
            id: ping.id,
            ping: ping.value,
            pong: 456,
            message: Bytes32::from(bytes),
        };

        entity.save();
    }
}

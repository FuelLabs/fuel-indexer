extern crate alloc;
use fuel_indexer_macros::indexer;

#[indexer(
    abi = "fuel-indexer-tests/contracts/fuel-indexer-test/out/debug/fuel-indexer-test-abi.json",
    namespace = "fuel_indexer_test",
    identifier = "index1",
    schema = "./../../../assets/fuel_indexer_test.graphql"
)]
mod fuel_indexer_test {
    fn function_one(ping: Ping) {
        Logger::info("function_one handling a Ping event.");

        let message: String = ping.message.into();

        let mut bytes: [u8; 32] = [0u8; 32];
        bytes.copy_from_slice(&message.as_bytes()[..32]);

        let entity = Message {
            id: ping.id,
            ping: ping.value,
            pong: 456,
            message: Bytes32::from(bytes),
        };

        entity.save();
    }
}

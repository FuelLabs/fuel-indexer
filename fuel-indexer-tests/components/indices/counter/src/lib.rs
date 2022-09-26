extern crate alloc;
use fuel_indexer_macros::indexer;

#[indexer(
    abi = "fuel-indexer-tests/contracts/fuel-indexer/out/debug/fuel-indexer-abi.json",
    namespace = "fuel_indexer",
    identifier = "index1",
    schema = "./../../../assets/fuel_indexer.graphql"
)]
mod fuel_indexer_test {
    fn handler_one(p: Ping) {
        Logger::info("handler_one handling a Ping event.");

        let mut bytes: [u8; 32] = [0u8; 32];

        bytes.copy_from_slice(&p.message.as_bytes()[..32]);

        let m = Message {
            id: p.id,
            ping: p.value,
            pong: 456,
            message: Bytes32::from(bytes),
        };

        m.save();
    }
}

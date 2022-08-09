extern crate alloc;
use fuel_indexer_macros::{graphql_schema, handler};
use fuel_indexer_schema::Bytes32;
use fuels_abigen_macro::wasm_abigen;

// IMPORTANT: This must match the namespace in your
graphql_schema!("composability_test", "schema/schema.graphql");
wasm_abigen!(
    no_name,
    "tests/e2e/composable-indexer-test/indexer/contracts/ping/out/debug/ping-abi.json"
);

#[handler]
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

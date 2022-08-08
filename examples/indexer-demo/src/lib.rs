extern crate alloc;
use fuel_indexer_macros::{graphql_schema, handler};
use fuels_abigen_macro::wasm_abigen;

graphql_schema!("demo_namespace", "schema/demo_schema.graphql");
wasm_abigen!(no_name, "examples/indexer-demo/contracts/indexer_demo.json");

#[handler]
fn function_one(event: LogEvent) {
    Logger::info("Callin' the event handler");
    let LogEvent {
        contract,
        rega,
        regb,
        ..
    } = event;

    let mut t1 = match Thing1::load(rega) {
        Some(t) => t,
        None => Thing1 {
            id: rega,
            account: Address::from(contract),
            count: 0,
        },
    };

    t1.count += regb;

    t1.save();
}

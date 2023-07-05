use std::path::Path;

use criterion::Criterion;
use fuel_indexer::{Manifest, Module};
use fuel_indexer_benches::{create_wasm_indexer_benchmark, WORKSPACE_ROOT};
use fuel_indexer_lib::manifest::ContractIds;

fn setup_default_indexer_manifest() -> Manifest {
    let schema_path = Path::new(WORKSPACE_ROOT)
        .join("benches/wasm_examples/assets/default_indexer/schema/default_indexer.schema.graphql")
        .as_path()
        .to_str()
        .unwrap()
        .to_string();
    let module_path = Path::new(WORKSPACE_ROOT)
        .parent()
        .unwrap()
        .join("target/wasm32-unknown-unknown/release/default_indexer.wasm")
        .as_path()
        .to_str()
        .unwrap()
        .to_string();
    let manifest = Manifest {
        namespace: "indexer_benchmarks".to_string(),
        identifier: "default_indexer".to_string(),
        graphql_schema: schema_path,
        module: Module::Wasm(module_path),
        abi: None,
        fuel_client: None,
        metrics: None,
        contract_id: ContractIds::Single(None),
        start_block: None,
        end_block: None,
        resumable: None,
    };

    manifest
}

// The `criterion_group!` macro requires that target functions have a name.
// Thus, each target function is actually a wrapper around a closure in
// order to prevent code duplication.

pub(crate) fn default_indexer_10_blocks_start_1(c: &mut Criterion) {
    let f = create_wasm_indexer_benchmark(1, 10, "default_indexer_10_blocks_start_1");
    f(c, setup_default_indexer_manifest())
}

pub(crate) fn default_indexer_100_blocks_start_1(c: &mut Criterion) {
    let f = create_wasm_indexer_benchmark(1, 100, "default_indexer_100_blocks_start_1");
    f(c, setup_default_indexer_manifest())
}

pub(crate) fn default_indexer_1000_blocks_start_1(c: &mut Criterion) {
    let f = create_wasm_indexer_benchmark(1, 1000, "default_indexer_1000_blocks_start_1");
    f(c, setup_default_indexer_manifest())
}

pub(crate) fn default_indexer_10_blocks_start_50000(c: &mut Criterion) {
    let f =
        create_wasm_indexer_benchmark(50000, 10, "default_indexer_10_blocks_start_50000");
    f(c, setup_default_indexer_manifest())
}

pub(crate) fn default_indexer_100_blocks_start_50000(c: &mut Criterion) {
    let f = create_wasm_indexer_benchmark(
        50000,
        100,
        "default_indexer_100_blocks_start_50000",
    );
    f(c, setup_default_indexer_manifest())
}

pub(crate) fn default_indexer_1000_blocks_start_50000(c: &mut Criterion) {
    let f = create_wasm_indexer_benchmark(
        50000,
        1000,
        "default_indexer_1000_blocks_start_50000",
    );
    f(c, setup_default_indexer_manifest())
}

pub(crate) fn default_indexer_10_blocks_start_200000(c: &mut Criterion) {
    let f = create_wasm_indexer_benchmark(
        200000,
        10,
        "default_indexer_10_blocks_start_200000",
    );
    f(c, setup_default_indexer_manifest())
}

pub(crate) fn default_indexer_100_blocks_start_200000(c: &mut Criterion) {
    let f = create_wasm_indexer_benchmark(
        200000,
        100,
        "default_indexer_100_blocks_start_200000",
    );
    f(c, setup_default_indexer_manifest())
}

pub(crate) fn default_indexer_1000_blocks_start_200000(c: &mut Criterion) {
    let f = create_wasm_indexer_benchmark(
        200000,
        1000,
        "default_indexer_1000_blocks_start_200000",
    );
    f(c, setup_default_indexer_manifest())
}

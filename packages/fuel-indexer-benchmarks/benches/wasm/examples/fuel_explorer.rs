use criterion::Criterion;
use fuel_indexer_benchmarks::create_wasm_indexer_benchmark;
use fuel_indexer_lib::{config::IndexerConfig, manifest::Manifest};

fn setup_fuel_explorer_manifest() -> Manifest {
    let manifest_str = r#"
namespace: indexer_benchmarks 
abi: ~
identifier: fuel_explorer
fuel_client: ~
graphql_schema: ../../examples/fuel-explorer/fuel-explorer/schema/fuel_explorer.schema.graphql
module:
  wasm: ../../target/wasm32-unknown-unknown/release/fuel_explorer.wasm
metrics: ~
contract_id: ~
start_block: ~
end_block: ~
resumable: ~
    "#;

    Manifest::try_from(manifest_str).unwrap()
}

// The `criterion_group!` macro requires that target functions have a name.
// Thus, each target function is actually a wrapper around a closure in
// order to prevent code duplication.

pub(crate) fn fuel_explorer_10_blocks_start_1(c: &mut Criterion) {
    let start_block = 1;
    let num_blocks = 10;
    let f = create_wasm_indexer_benchmark(
        start_block,
        num_blocks,
        "fuel_explorer_10_blocks_start_1",
    );
    f(c, setup_fuel_explorer_manifest(), IndexerConfig::default())
}

pub(crate) fn fuel_explorer_100_blocks_start_1(c: &mut Criterion) {
    let start_block = 1;
    let num_blocks = 100;
    let f = create_wasm_indexer_benchmark(
        start_block,
        num_blocks,
        "fuel_explorer_100_blocks_start_1",
    );
    f(c, setup_fuel_explorer_manifest(), IndexerConfig::default())
}

pub(crate) fn fuel_explorer_1000_blocks_start_1(c: &mut Criterion) {
    let start_block = 1;
    let num_blocks = 1000;
    let f = create_wasm_indexer_benchmark(
        start_block,
        num_blocks,
        "fuel_explorer_1000_blocks_start_1",
    );
    f(c, setup_fuel_explorer_manifest(), IndexerConfig::default())
}

pub(crate) fn fuel_explorer_10_blocks_start_50000(c: &mut Criterion) {
    let start_block = 50000;
    let num_blocks = 10;
    let f = create_wasm_indexer_benchmark(
        start_block,
        num_blocks,
        "fuel_explorer_10_blocks_start_50000",
    );
    f(c, setup_fuel_explorer_manifest(), IndexerConfig::default())
}

pub(crate) fn fuel_explorer_100_blocks_start_50000(c: &mut Criterion) {
    let start_block = 50000;
    let num_blocks = 100;
    let f = create_wasm_indexer_benchmark(
        start_block,
        num_blocks,
        "fuel_explorer_100_blocks_start_50000",
    );
    f(c, setup_fuel_explorer_manifest(), IndexerConfig::default())
}

pub(crate) fn fuel_explorer_1000_blocks_start_50000(c: &mut Criterion) {
    let start_block = 50000;
    let num_blocks = 1000;
    let f = create_wasm_indexer_benchmark(
        start_block,
        num_blocks,
        "fuel_explorer_1000_blocks_start_50000",
    );
    f(c, setup_fuel_explorer_manifest(), IndexerConfig::default())
}

pub(crate) fn fuel_explorer_10_blocks_start_200000(c: &mut Criterion) {
    let start_block = 200000;
    let num_blocks = 10;
    let f = create_wasm_indexer_benchmark(
        start_block,
        num_blocks,
        "fuel_explorer_10_blocks_start_200000",
    );
    f(c, setup_fuel_explorer_manifest(), IndexerConfig::default())
}

pub(crate) fn fuel_explorer_100_blocks_start_200000(c: &mut Criterion) {
    let start_block = 200000;
    let num_blocks = 100;
    let f = create_wasm_indexer_benchmark(
        start_block,
        num_blocks,
        "fuel_explorer_100_blocks_start_200000",
    );
    f(c, setup_fuel_explorer_manifest(), IndexerConfig::default())
}

pub(crate) fn fuel_explorer_1000_blocks_start_200000(c: &mut Criterion) {
    let start_block = 200000;
    let num_blocks = 1000;
    let f = create_wasm_indexer_benchmark(
        start_block,
        num_blocks,
        "fuel_explorer_1000_blocks_start_200000",
    );
    f(c, setup_fuel_explorer_manifest(), IndexerConfig::default())
}

pub(crate) fn fuel_explorer_10_blocks_start_1000000(c: &mut Criterion) {
    let start_block = 1000000;
    let num_blocks = 10;
    let f = create_wasm_indexer_benchmark(
        start_block,
        num_blocks,
        "fuel_explorer_10_blocks_start_1000000",
    );
    f(c, setup_fuel_explorer_manifest(), IndexerConfig::default())
}

pub(crate) fn fuel_explorer_100_blocks_start_1000000(c: &mut Criterion) {
    let start_block = 1000000;
    let num_blocks = 100;
    let f = create_wasm_indexer_benchmark(
        start_block,
        num_blocks,
        "fuel_explorer_100_blocks_start_1000000",
    );
    f(c, setup_fuel_explorer_manifest(), IndexerConfig::default())
}

// Due to the size of blocks at this point in the chain, metering needs to be disabled
// to process this amount of blocks for benchmarking.
pub(crate) fn fuel_explorer_1000_blocks_start_1000000(c: &mut Criterion) {
    let start_block = 1000000;
    let num_blocks = 1000;
    let f = create_wasm_indexer_benchmark(
        start_block,
        num_blocks,
        "fuel_explorer_1000_blocks_start_1000000",
    );
    f(
        c,
        setup_fuel_explorer_manifest(),
        IndexerConfig {
            metering_points: None,
            ..Default::default()
        },
    )
}

pub(crate) fn fuel_explorer_10_blocks_start_1500000(c: &mut Criterion) {
    let start_block = 1500000;
    let num_blocks = 10;
    let f = create_wasm_indexer_benchmark(
        start_block,
        num_blocks,
        "fuel_explorer_10_blocks_start_1500000",
    );
    f(c, setup_fuel_explorer_manifest(), IndexerConfig::default())
}

pub(crate) fn fuel_explorer_100_blocks_start_1500000(c: &mut Criterion) {
    let start_block = 1500000;
    let num_blocks = 100;
    let f = create_wasm_indexer_benchmark(
        start_block,
        num_blocks,
        "fuel_explorer_100_blocks_start_1500000",
    );
    f(c, setup_fuel_explorer_manifest(), IndexerConfig::default())
}

// Due to the size of blocks at this point in the chain, metering needs to be disabled
// to process this amount of blocks for benchmarking.
pub(crate) fn fuel_explorer_1000_blocks_start_1500000(c: &mut Criterion) {
    let start_block = 1500000;
    let num_blocks = 1000;
    let f = create_wasm_indexer_benchmark(
        start_block,
        num_blocks,
        "fuel_explorer_1000_blocks_start_1500000",
    );
    f(
        c,
        setup_fuel_explorer_manifest(),
        IndexerConfig {
            metering_points: None,
            ..Default::default()
        },
    )
}

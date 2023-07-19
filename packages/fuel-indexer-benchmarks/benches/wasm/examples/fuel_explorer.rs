use criterion::Criterion;
use fuel_indexer_benchmarks::{create_wasm_indexer_benchmark, create_wasm_manifest};
use fuel_indexer_lib::{config::IndexerConfig, manifest::Manifest};

fn setup_fuel_explorer_manifest() -> Manifest {
    create_wasm_manifest(
        "indexer_benchmarks",
        "fuel_explorer",
        "examples/fuel-explorer/fuel-explorer/schema/fuel_explorer.schema.graphql",
        "target/wasm32-unknown-unknown/release/fuel_explorer.wasm",
    )
}

// The `criterion_group!` macro requires that target functions have a name.
// Thus, each target function is actually a wrapper around a closure in
// order to prevent code duplication.

pub(crate) fn fuel_explorer_10_blocks_start_1(c: &mut Criterion) {
    let f = create_wasm_indexer_benchmark(1, 10, "fuel_explorer_10_blocks_start_1");
    f(c, setup_fuel_explorer_manifest(), IndexerConfig::default())
}

pub(crate) fn fuel_explorer_100_blocks_start_1(c: &mut Criterion) {
    let f = create_wasm_indexer_benchmark(1, 100, "fuel_explorer_100_blocks_start_1");
    f(c, setup_fuel_explorer_manifest(), IndexerConfig::default())
}

pub(crate) fn fuel_explorer_1000_blocks_start_1(c: &mut Criterion) {
    let f = create_wasm_indexer_benchmark(1, 1000, "fuel_explorer_1000_blocks_start_1");
    f(c, setup_fuel_explorer_manifest(), IndexerConfig::default())
}

pub(crate) fn fuel_explorer_10_blocks_start_50000(c: &mut Criterion) {
    let f =
        create_wasm_indexer_benchmark(50000, 10, "fuel_explorer_10_blocks_start_50000");
    f(c, setup_fuel_explorer_manifest(), IndexerConfig::default())
}

pub(crate) fn fuel_explorer_100_blocks_start_50000(c: &mut Criterion) {
    let f =
        create_wasm_indexer_benchmark(50000, 100, "fuel_explorer_100_blocks_start_50000");
    f(c, setup_fuel_explorer_manifest(), IndexerConfig::default())
}

pub(crate) fn fuel_explorer_1000_blocks_start_50000(c: &mut Criterion) {
    let f = create_wasm_indexer_benchmark(
        50000,
        1000,
        "fuel_explorer_1000_blocks_start_50000",
    );
    f(c, setup_fuel_explorer_manifest(), IndexerConfig::default())
}

pub(crate) fn fuel_explorer_10_blocks_start_200000(c: &mut Criterion) {
    let f =
        create_wasm_indexer_benchmark(200000, 10, "fuel_explorer_10_blocks_start_200000");
    f(c, setup_fuel_explorer_manifest(), IndexerConfig::default())
}

pub(crate) fn fuel_explorer_100_blocks_start_200000(c: &mut Criterion) {
    let f = create_wasm_indexer_benchmark(
        200000,
        100,
        "fuel_explorer_100_blocks_start_200000",
    );
    f(c, setup_fuel_explorer_manifest(), IndexerConfig::default())
}

pub(crate) fn fuel_explorer_1000_blocks_start_200000(c: &mut Criterion) {
    let f = create_wasm_indexer_benchmark(
        200000,
        1000,
        "fuel_explorer_1000_blocks_start_200000",
    );
    f(c, setup_fuel_explorer_manifest(), IndexerConfig::default())
}

pub(crate) fn fuel_explorer_10_blocks_start_1000000(c: &mut Criterion) {
    let f = create_wasm_indexer_benchmark(
        1000000,
        10,
        "fuel_explorer_10_blocks_start_1000000",
    );
    f(c, setup_fuel_explorer_manifest(), IndexerConfig::default())
}

pub(crate) fn fuel_explorer_100_blocks_start_1000000(c: &mut Criterion) {
    let f = create_wasm_indexer_benchmark(
        1000000,
        100,
        "fuel_explorer_100_blocks_start_1000000",
    );
    f(c, setup_fuel_explorer_manifest(), IndexerConfig::default())
}

pub(crate) fn fuel_explorer_1000_blocks_start_1000000(c: &mut Criterion) {
    let f = create_wasm_indexer_benchmark(
        1000000,
        1000,
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
    let f = create_wasm_indexer_benchmark(
        1500000,
        10,
        "fuel_explorer_10_blocks_start_1500000",
    );
    f(c, setup_fuel_explorer_manifest(), IndexerConfig::default())
}

pub(crate) fn fuel_explorer_100_blocks_start_1500000(c: &mut Criterion) {
    let f = create_wasm_indexer_benchmark(
        1500000,
        100,
        "fuel_explorer_100_blocks_start_1500000",
    );
    f(c, setup_fuel_explorer_manifest(), IndexerConfig::default())
}

pub(crate) fn fuel_explorer_1000_blocks_start_1500000(c: &mut Criterion) {
    let f = create_wasm_indexer_benchmark(
        1500000,
        1000,
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

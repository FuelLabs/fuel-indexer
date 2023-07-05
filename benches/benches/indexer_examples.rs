mod wasm_examples;

use criterion::{criterion_group, criterion_main};

use wasm_examples::{default_indexer::*, fuel_explorer::*};

criterion_group!(
    fuel_explorer_benches,
    fuel_explorer_10_blocks_start_1,
    fuel_explorer_100_blocks_start_1,
    fuel_explorer_1000_blocks_start_1,
    fuel_explorer_10_blocks_start_50000,
    fuel_explorer_100_blocks_start_50000,
    fuel_explorer_1000_blocks_start_50000,
    fuel_explorer_10_blocks_start_200000,
    fuel_explorer_100_blocks_start_200000,
    fuel_explorer_1000_blocks_start_200000
);

criterion_group!(
    default_indexer_benches,
    default_indexer_10_blocks_start_1,
    default_indexer_100_blocks_start_1,
    default_indexer_1000_blocks_start_1,
    default_indexer_10_blocks_start_50000,
    default_indexer_100_blocks_start_50000,
    default_indexer_1000_blocks_start_50000,
    default_indexer_10_blocks_start_200000,
    default_indexer_100_blocks_start_200000,
    default_indexer_1000_blocks_start_200000
);

criterion_main!(fuel_explorer_benches, default_indexer_benches);

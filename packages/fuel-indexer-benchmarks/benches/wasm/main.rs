mod examples;

use crate::examples::fuel_explorer::*;
use criterion::{criterion_group, criterion_main};

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
    fuel_explorer_1000_blocks_start_200000,
    fuel_explorer_10_blocks_start_1000000,
    fuel_explorer_100_blocks_start_1000000,
    fuel_explorer_1000_blocks_start_1000000,
    fuel_explorer_10_blocks_start_1500000,
    fuel_explorer_100_blocks_start_1500000,
    fuel_explorer_1000_blocks_start_1500000,
);

criterion_main!(fuel_explorer_benches);

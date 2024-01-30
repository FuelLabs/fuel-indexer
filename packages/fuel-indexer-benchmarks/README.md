# fuel-indexer-benchmarks

An internal crate that seeks to answer the age-old question: "How fast are things, really?"

## Setup

Before running the benchmarks, you should build the `fuel-explorer` WASM example 
and place it in the `target/wasm32-unknown-unknown/release` folder at the root of the repository. 
You can do this automatically by running the `build_test_wasm_module` script found 
in the `/scripts/utils` folder of the repo root.

## Usage

### `criterion` benchmarking

Ensure that you are in the `fuel-indexer-benchmarks` directory and then run `cargo bench`. The
compile time for benchmarks is a bit longer than that of normal compilation, so it may take longer 
than one would expect. Eventually, timings will be logged to the terminal, showing you the minimum, 
average, and maximum times for each benchmark; they will also be saved for comparison against future 
runs in `target/criterion`. Additionally, an HTML file with plots and statistics can be found at 
`target/criterion/report/index.html`.

### QA suite

```bash
cargo run -p fuel-indexer-benchmarks --bin qa -- --network beta-5.fuel.network
```

### Comparing Branches

You can compare branches by switching to the base branch, running `cargo bench`, then switching to your
desired branch, and running `cargo bench` again. `criterion` will use the results from the last benchmarking 
run and report any statistically significant changes.
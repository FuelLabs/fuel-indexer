use criterion::Criterion;
use fuel_core_client::client::FuelClient;
use fuel_indexer::{
    executor::retrieve_blocks_from_node, prelude::fuel::BlockData, Executor,
    IndexerConfig, Manifest, WasmIndexExecutor,
};
use fuel_indexer_database::IndexerConnectionPool;
use fuel_indexer_lib::config::DatabaseConfig;
use fuel_indexer_tests::fixtures::TestPostgresDb;
use std::str::FromStr;

/// Location of Fuel node to be used for block retrieval.
pub const NODE_URL: &str = "beta-3.fuel.network:80";

/// Cargo workspace root; used to ensure correct file paths.
pub const WORKSPACE_ROOT: &str = env!("CARGO_MANIFEST_DIR");

/// Retrieve blocks from the Fuel node located at `NODE_URL`.
async fn get_blocks(start_cursor: u64, num_blocks: usize) -> Result<Vec<BlockData>, ()> {
    let client = FuelClient::from_str(NODE_URL)
        .unwrap_or_else(|e| panic!("Node connection failed: {e}."));
    let next_cursor = Some(start_cursor.to_string());
    let (blocks, _) = retrieve_blocks_from_node(
        &client,
        num_blocks,
        &next_cursor,
        Some(start_cursor + num_blocks as u64),
        "",
    )
    .await
    .expect("Could not retrieve blocks from node");

    Ok(blocks)
}

/// Create WASM executor for use in a benchmarking function.
async fn setup_wasm_executor(
    mut config: IndexerConfig,
    manifest: Manifest,
    db_url: String,
    pool: IndexerConnectionPool,
) -> Result<WasmIndexExecutor, ()> {
    config.database = DatabaseConfig::from_str(&db_url).unwrap();
    let schema_version = manifest.graphql_schema_content().unwrap().version().to_string();
    let executor = WasmIndexExecutor::new(
        &config,
        &manifest,
        manifest.module_bytes().unwrap(),
        pool,
        schema_version
    )
    .await
    .expect("Could not setup WASM executor");
    Ok(executor)
}

/// Create a function for use in benchmarking a WASM indexer.
/// Staring from `start_block`, the function retrieves an amount
/// of blocks equal to `num_blocks` and then passes it into the indexer.
pub fn create_wasm_indexer_benchmark(
    start_block: u64,
    num_blocks: usize,
    name: &str,
) -> impl Fn(&mut Criterion, Manifest, IndexerConfig) + '_ {
    move |c: &mut Criterion, manifest: Manifest, config: IndexerConfig| {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let blocks = rt.block_on(get_blocks(start_block, num_blocks)).unwrap();
        c.bench_function(name, move |b| {
            b.iter_batched(
                // This setup function is run prior to each iteration of
                // the benchmark; this ensures that there is a fresh WASM
                // executor and isolated test database for each run.
                || {
                    rt.block_on(async {
                        let test_db = TestPostgresDb::new().await.unwrap();
                        let executor = setup_wasm_executor(
                            config.clone(),
                            manifest.clone(),
                            test_db.url.clone(),
                            IndexerConnectionPool::Postgres(test_db.pool.clone()),
                        )
                        .await
                        .unwrap();
                        (executor, blocks.clone())
                    })
                },
                |(mut ex, blocks)| rt.block_on(ex.handle_events(blocks)),
                criterion::BatchSize::SmallInput,
            )
        });
    }
}

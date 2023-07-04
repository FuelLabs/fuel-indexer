use criterion::{black_box, criterion_group, criterion_main, Criterion};

use fuel_indexer::{Executor, IndexerConfig, Manifest, WasmIndexExecutor};
use std::{fs::File, io::Read};

fn criterion_benchmark(c: &mut Criterion) {
    if let Ok(current_dir) = std::env::current_dir() {
        println!("Current directory: {}", current_dir.display());
    }

    if let Ok(mut current_dir) = std::env::current_dir() {
        current_dir.pop();
        current_dir.pop();
        if let Err(e) = std::env::set_current_dir(current_dir) {
            eprintln!("Failed to change directory: {}", e);
        }
    }

    let manifest = Manifest::from_file(
        "packages/fuel-indexer-tests/components/indices/simple-wasm/simple_wasm.yaml",
    )
    .unwrap();

    let wasm_bytes = match &manifest.module {
        fuel_indexer_lib::manifest::Module::Wasm(ref module) => {
            let mut bytes = Vec::<u8>::new();
            let mut file = File::open(module).unwrap();
            file.read_to_end(&mut bytes).unwrap();
            bytes
        }
        _ => panic!("unexpected"),
    };

    //let rt = tokio::runtime::Runtime::new().unwrap();

    let rt = tokio::runtime::Builder::new_current_thread().enable_time().enable_io()
        .build()
        .unwrap();

    let mut group = c.benchmark_group("executor");
    for t in [false] {
        group.bench_function(&format!("metering={t}"), |b| {
            b.to_async(&rt).iter_custom(|iters| {
                let manifest = manifest.clone();
                let wasm_bytes = wasm_bytes.clone();

                let metering_points = if t { Some(9_000_000_000_000) } else { None };

                async move {
                    let mut config = IndexerConfig::default();
                    config.indexer_handler_timeout = 1000;

                    let mut metering_executor = WasmIndexExecutor::new(
                        &config,
                        &manifest,
                        wasm_bytes,
                        metering_points,
                    )
                    .await
                    .unwrap();

                    let blocks: Vec<fuel_indexer_types::fuel::BlockData> = vec![];

                    let start = std::time::Instant::now();
                    for _ in 0..iters {
                        metering_executor
                            .handle_events(black_box(blocks.clone()))
                            .await
                            .unwrap()
                    }
                    start.elapsed()
                }
            })
        });
    }
    group.finish();
}

criterion_group! {
    name = benches;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default().sample_size(10).measurement_time(std::time::Duration::from_secs(5));
    targets = criterion_benchmark
}
criterion_main!(benches);

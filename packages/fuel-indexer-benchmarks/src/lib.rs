#![cfg_attr(feature = "benchmarks", feature(test))]

#[cfg(feature = "benchmarks")]
#[cfg(test)]
extern crate test;

#[cfg(feature = "benchmarks")]
#[cfg(test)]
mod tests {
    use test::Bencher;

    use fuel_indexer::{Executor, IndexerConfig, Manifest, WasmIndexExecutor};
    use std::{fs::File, io::Read};

    fn make_executor_metering_benchmark(b: &mut Bencher, metering_points: Option<u64>) {
        if let Ok(mut current_dir) = std::env::current_dir() {
            if current_dir.ends_with("fuel-indexer-benchmarks") {
                current_dir.pop();
                current_dir.pop();
            }

            if let Err(e) = std::env::set_current_dir(current_dir) {
                eprintln!("Failed to change directory: {}", e);
            }
        }

        let manifest = Manifest::from_file(
            "packages/fuel-indexer-tests/components/indices/fuel-indexer-test/fuel_indexer_test.yaml",
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

        let rt = tokio::runtime::Runtime::new().unwrap();

        let mut executor = rt.block_on(async {
            let mut config = IndexerConfig::default();
            config.indexer_handler_timeout = 60;

            let manifest = manifest.clone();
            let wasm_bytes = wasm_bytes.clone();

            // let pool = fuel_indexer_database::IndexerConnectionPool::connect(
            //     &config.database.to_string(),
            // )
            // .await
            // .unwrap();

            WasmIndexExecutor::new(&config, &manifest, wasm_bytes, metering_points)
                .await
                .unwrap()
        });

        let blocks: Vec<fuel_indexer_types::fuel::BlockData> = vec![];
        for _ in 0..16 {
            b.iter(|| {
                rt.block_on(executor.handle_events(blocks.clone())).unwrap();
            })
        }
    }

    #[bench]
    fn executor_with_metering(b: &mut Bencher) {
        make_executor_metering_benchmark(b, Some(9_000_000_000_000));
    }

    #[bench]
    fn executor_no_metering(b: &mut Bencher) {
        make_executor_metering_benchmark(b, None);
    }
}

#[cfg(test)]
mod tests {
    use fuel_indexer::executor::*;
    use fuel_tx::Receipt;
    use fuels_abigen_macro::abigen;
    use fuels_core::{abi_encoder::ABIEncoder, Tokenizable};
    use sqlx::{Connection, Row};

    const MANIFEST: &str = include_str!("test_data/manifest.yaml");
    const BAD_MANIFEST: &str = include_str!("test_data/bad_manifest.yaml");
    const BAD_WASM_BYTES: &[u8] = include_bytes!("test_data/bad_simple_wasm.wasm");
    const WASM_BYTES: &[u8] = include_bytes!("test_data/simple_wasm.wasm");

    abigen!(MyContract, "fuel-indexer/src/test_data/contracts-abi.json");

    #[derive(Debug)]
    struct Thing1 {
        id: i64,
        account: String,
    }

    #[tokio::test]
    async fn test_postgres() {
        let database_url = "postgres://postgres:my-secret@127.0.0.1:5432";

        do_test(database_url).await;

        let mut conn = sqlx::PgConnection::connect(database_url)
            .await
            .expect("Database connection failed!");

        let row = sqlx::query("select id,account from test_namespace.thing1 where id = 1020;")
            .fetch_one(&mut conn)
            .await
            .expect("Database query failed");

        let id = row.get(0);
        let account = row.get(1);

        let data = Thing1 { id, account };

        assert_eq!(data.id, 1020);
        assert_eq!(
            data.account,
            "afafafafafafafafafafafafafafafafafafafafafafafafafafafafafafafaf"
        );
    }

    #[tokio::test]
    async fn test_sqlite() {
        let workspace_root = env!("CARGO_MANIFEST_DIR");
        let database_url = format!("sqlite://{}/test.db", workspace_root);

        do_test(&database_url).await;

        let mut conn = sqlx::SqliteConnection::connect(&database_url)
            .await
            .expect("Database connection failed!");

        let row = sqlx::query("select id,account from thing1 where id = 1020;")
            .fetch_one(&mut conn)
            .await
            .expect("Database query failed");

        let id = row.get(0);
        let account = row.get(1);

        let data = Thing1 { id, account };

        assert_eq!(data.id, 1020);
        assert_eq!(
            data.account,
            "afafafafafafafafafafafafafafafafafafafafafafafafafafafafafafafaf"
        );
    }

    async fn do_test(database_url: &str) {
        let manifest: Manifest = serde_yaml::from_str(MANIFEST).expect("Bad yaml file.");
        let bad_manifest: Manifest = serde_yaml::from_str(BAD_MANIFEST).expect("Bad yaml file.");

        let executor =
            WasmIndexExecutor::new(database_url.to_string(), bad_manifest, BAD_WASM_BYTES).await;
        match executor {
            Err(IndexerError::MissingHandler) => (),
            e => panic!("Expected missing handler error {:#?}", e),
        }

        let executor = WasmIndexExecutor::new(database_url.to_string(), manifest, WASM_BYTES).await;
        assert!(executor.is_ok());

        let mut executor = executor.unwrap();

        let evt1 = SomeEvent {
            id: 1020,
            account: [0xaf; 32],
        };
        let evt2 = AnotherEvent {
            id: 100,
            account: [0x5a; 32],
            hash: [0x43; 32],
        };

        let some_event = ABIEncoder::new()
            .encode(&[evt1.into_token()])
            .expect("Failed to encode");
        let another_event = ABIEncoder::new()
            .encode(&[evt2.into_token()])
            .expect("Failed to encode");

        let result = executor
            .handle_events(vec![BlockData {
                height: 0,
                transactions: vec![
                    vec![
                        Receipt::Call {
                            id: [0u8; 32].into(),
                            to: [0u8; 32].into(),
                            amount: 400,
                            asset_id: [0u8; 32].into(),
                            gas: 4,
                            param1: 2048508220,
                            param2: 0,
                            pc: 0,
                            is: 0,
                        },
                        Receipt::ReturnData {
                            id: [0u8; 32].into(),
                            ptr: 2342143,
                            len: some_event.len() as u64,
                            digest: [0u8; 32].into(),
                            data: some_event,
                            pc: 0,
                            is: 0,
                        },
                    ],
                    vec![
                        Receipt::Call {
                            id: [0u8; 32].into(),
                            to: [0u8; 32].into(),
                            amount: 400,
                            asset_id: [0u8; 32].into(),
                            gas: 4,
                            param1: 2379805026,
                            param2: 0,
                            pc: 0,
                            is: 0,
                        },
                        Receipt::ReturnData {
                            id: [0u8; 32].into(),
                            ptr: 2342143,
                            len: another_event.len() as u64,
                            digest: [0u8; 32].into(),
                            data: another_event,
                            pc: 0,
                            is: 0,
                        },
                    ],
                ],
            }])
            .await;
        assert!(result.is_ok());
    }
}

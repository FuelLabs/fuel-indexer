extern crate alloc;

#[cfg(feature = "postgres")]
mod tests {
    use fuel_core::service::{Config, FuelService};
    use fuel_gql_client::client::FuelClient;
    use fuel_indexer::{IndexerConfig, IndexerService, Manifest};
    use fuel_vm::{consts::*, prelude::*};

    const DATABASE_URL: &str = "postgres://postgres:my-secret@127.0.0.1:5432";
    const MANIFEST: &str = include_str!("./test_data/demo_manifest.yaml");

    #[allow(clippy::iter_cloned_collect)]
    fn create_log_transaction(rega: u16, regb: u16) -> Transaction {
        #[allow(clippy::iter_cloned_collect)]
        let script = [
            Opcode::ADDI(0x10, REG_ZERO, rega),
            Opcode::ADDI(0x11, REG_ZERO, regb),
            Opcode::LOG(0x10, 0x11, REG_ZERO, REG_ZERO),
            Opcode::LOG(0x11, 0x12, REG_ZERO, REG_ZERO),
            Opcode::RET(REG_ONE),
        ]
        .iter()
        .copied()
        .collect();

        let byte_price = 0;
        let gas_price = 0;
        let gas_limit = 1_000_000;
        let maturity = 0;
        Transaction::script(
            gas_price,
            gas_limit,
            byte_price,
            maturity,
            script,
            vec![],
            vec![],
            vec![],
            vec![],
        )
    }

    #[tokio::test]
    async fn test_blocks() {
        let srv = FuelService::new_node(Config::local_node()).await.unwrap();
        let client = FuelClient::from(srv.bound_address);
        // submit tx
        let _ = client.submit(&create_log_transaction(0xca, 0xba)).await;
        let _ = client.submit(&create_log_transaction(0xfa, 0x4f)).await;
        let _ = client.submit(&create_log_transaction(0x33, 0x11)).await;

        let dir = std::env::current_dir().unwrap();
        let test_data = dir.join("tests/test_data");

        let config = IndexerConfig {
            fuel_node_addr: srv.bound_address,
            database_url: DATABASE_URL.to_string(),
            listen_endpoint: "0.0.0.0:9999".parse().unwrap(),
        };

        let mut indexer_service = IndexerService::new(config).unwrap();

        let mut manifest: Manifest = serde_yaml::from_str(MANIFEST).expect("Bad yaml file");

        manifest.graphql_schema = test_data
            .join(manifest.graphql_schema)
            .display()
            .to_string();
        manifest.wasm_module = Some(
            test_data
                .join(manifest.wasm_module.unwrap())
                .display()
                .to_string(),
        );

        indexer_service
            .add_wasm_indexer(manifest, true)
            .expect("Failed to initialize indexer");

        indexer_service.run().await;
    }
}

extern crate alloc;

#[cfg(not_yet)]
#[cfg(test)]
mod tests {
    use fuel_core::service::{Config, FuelService};
    use fuel_crypto::SecretKey;
    use fuel_gql_client::client::FuelClient;
    use fuel_indexer::{IndexerConfig, IndexerService, Manifest};
    use fuel_indexer_lib::config::{DatabaseConfig, FuelNodeConfig, GraphQLConfig};
    use fuel_vm::{consts::*, prelude::*};
    use fuels::prelude::{Contract, LocalWallet, Provider, TxParameters};
    use fuels::signers::wallet::Wallet;
    use fuels_abigen_macro::abigen;
    use rand::{rngs::StdRng, SeedableRng};
    use std::path::Path;

    const MANIFEST: &str = include_str!("./test_data/manifest.yaml");
    const WORKSPACE_DIR: &str = env!("CARGO_MANIFEST_DIR");

    abigen!(Simple, "./fuel-indexer/tests/test_data/contracts-abi.json");

    fn tx_params() -> TxParameters {
        let gas_price = 0;
        let gas_limit = 1_000_000;
        let byte_price = 0;
        TxParameters::new(Some(gas_price), Some(gas_limit), Some(byte_price), None)
    }

    #[tokio::test]
    async fn test_blocks() {
        let workdir = Path::new(WORKSPACE_DIR);
        let srv = FuelService::new_node(Config::local_node()).await.unwrap();

        let mut rng = StdRng::seed_from_u64(10);

        let provider = Provider::connect(srv.bound_address).await.unwrap();

        let secret = SecretKey::random(&mut rng);
        let wallet = LocalWallet::new_from_private_key(secret, Some(provider.clone()));

        let p = workdir.join("./tests/test_data/contracts.bin");
        let path = p.as_os_str().to_str().unwrap();
        let _compiled = Contract::load_sway_contract(path).unwrap();

        let contract_id = Contract::deploy(path, &wallet, tx_params()).await.unwrap();

        let contract: Simple = Simple::new(contract_id.to_string(), wallet);
        let _ = contract.gimme_someevent(78).call().await;
        let _ = contract.gimme_anotherevent(899).call().await;

        let dir = std::env::current_dir().unwrap();
        let test_data = dir.join("tests/test_data");

        let config = IndexerConfig {
            fuel_node: FuelNodeConfig::from(srv.bound_address),
            database: DatabaseConfig::Postgres {
                user: "postgres".into(),
                password: Some("my-secret".into()),
                host: "127.0.0.1".into(),
                port: "5432".into(),
                database: None,
            },
            graphql_api: GraphQLConfig::default(),
        };

        let mut indexer_service = IndexerService::new(config).await.unwrap();

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
            .add_indexer(manifest, true)
            .await
            .expect("Failed to initialize indexer");

        indexer_service.run().await;
    }
}

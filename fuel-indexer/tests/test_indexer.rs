extern crate alloc;

#[cfg(test)]
mod tests {
    use fuel_crypto::SecretKey;
    use fuel_indexer::{IndexerConfig, IndexerService, Manifest};
    use fuel_indexer_lib::config::{DatabaseConfig, FuelNodeConfig, GraphQLConfig};
    use fuel_vm::{consts::*, prelude::*};
    use fuels::node::{
        chain_config::{ChainConfig, StateConfig},
        service::DbType,
    };
    use fuels::prelude::{
        setup_single_asset_coins, setup_test_client, AssetId, Config, Contract, LocalWallet,
        Provider, TxParameters, DEFAULT_COIN_AMOUNT,
    };
    use fuels::signers::{wallet::Wallet, Signer};
    use fuels_abigen_macro::abigen;
    use rand::{rngs::StdRng, SeedableRng};
    use std::path::Path;

    const MANIFEST: &str = include_str!("./test_data/manifest.yaml");
    const WORKSPACE_DIR: &str = env!("CARGO_MANIFEST_DIR");
    const FUEL_NODE_ADDR: &str = "0.0.0.0:4000";

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

        let p = workdir.join("./tests/test_data/wallet.json");
        let path = p.as_os_str().to_str().unwrap();
        let mut wallet =
            LocalWallet::load_keystore(&path, "password", None).expect("Could load keys");

        let p = workdir.join("./tests/test_data/contracts.bin");
        let path = p.as_os_str().to_str().unwrap();
        let _compiled = Contract::load_sway_contract(path).unwrap();

        let number_of_coins = 11;
        let asset_id = AssetId::zeroed();
        let coins = setup_single_asset_coins(
            wallet.address(),
            asset_id,
            number_of_coins,
            DEFAULT_COIN_AMOUNT,
        );

        let config = Config {
            chain_conf: ChainConfig {
                initial_state: Some(StateConfig {
                    ..StateConfig::default()
                }),
                ..ChainConfig::local_testnet()
            },
            database_type: DbType::InMemory,
            utxo_validation: false,
            addr: FUEL_NODE_ADDR.parse().unwrap(),
            ..Config::local_node()
        };

        let (client, _) = setup_test_client(coins, config).await;

        let provider = Provider::new(client);

        wallet.set_provider(provider.clone());

        let contract_id = Contract::deploy(path, &wallet, tx_params()).await.unwrap();

        let contract: Simple = Simple::new(contract_id.to_string(), wallet);
        let _ = contract.gimme_someevent(78).call().await;
        let _ = contract.gimme_anotherevent(899).call().await;

        let dir = std::env::current_dir().unwrap();
        let test_data = dir.join("tests/test_data");

        let config = IndexerConfig {
            fuel_node: FuelNodeConfig::from(
                FUEL_NODE_ADDR.parse::<std::net::SocketAddr>().unwrap(),
            ),
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

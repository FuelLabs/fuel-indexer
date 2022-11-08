pub mod assets {
    pub const MANIFEST: &str = include_str!("./../assets/simple_wasm.yaml");
    pub const BAD_MANIFEST: &str = include_str!("./../assets/bad_simple_wasm.yaml");
    pub const BAD_WASM_BYTES: &[u8] = include_bytes!("./../assets/bad_simple_wasm.wasm");
    pub const WASM_BYTES: &[u8] = include_bytes!("./../assets/simple_wasm.wasm");
}

pub mod defaults {
    use std::time::Duration;

    pub const FUEL_NODE_ADDR: &str = "127.0.0.1:4000";
    pub const FUEL_NODE_HOST: &str = "127.0.0.1";
    pub const FUEL_NODE_PORT: &str = "4000";
    pub const WEB_API_ADDR: &str = "127.0.0.1:8000";
    pub const PING_CONTRACT_ID: &str =
        "68518c3ba3768c863e0d945aa18249f9516d3aa1338083ba79467aa393de109c";
    pub const TRANSFER_BASE_ASSET_ID: &str =
        "0000000000000000000000000000000000000000000000000000000000000000";
    pub const SLEEP: Duration = Duration::from_secs(60 * 60 * 10);
    pub const WALLET_PASSWORD: &str = "password";
    pub const INDEXED_EVENT_WAIT: u64 = 5;
}

pub mod fixtures {

    use fuel_indexer_database::IndexerConnectionPool;
    use fuels::prelude::TxParameters;
    use sqlx::{pool::Pool, Postgres};

    pub async fn postgres_connection_pool(db_url: &str) -> Pool<Postgres> {
        match IndexerConnectionPool::connect(db_url).await.unwrap() {
            IndexerConnectionPool::Postgres(p) => p,
            _ => panic!("Should be postgres."),
        }
    }

    pub fn http_client() -> reqwest::Client {
        reqwest::Client::new()
    }

    pub fn tx_params() -> TxParameters {
        let gas_price = 0;
        let gas_limit = 1_000_000;
        let byte_price = 0;
        TxParameters::new(Some(gas_price), Some(gas_limit), Some(byte_price))
    }
}

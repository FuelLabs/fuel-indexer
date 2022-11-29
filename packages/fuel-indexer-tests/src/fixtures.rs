use crate::WORKSPACE_ROOT;
use fuel_indexer_database::IndexerConnectionPool;
use fuel_indexer_lib::config::DatabaseConfig;
use fuels::prelude::TxParameters;
use sqlx::{pool::PoolConnection, Postgres, Sqlite};

pub async fn postgres_connection() -> PoolConnection<Postgres> {
    let config = DatabaseConfig::Postgres {
        user: "postgres".into(),
        password: "my-secret".into(),
        host: "127.0.0.1".into(),
        port: "5432".into(),
        database: "postgres".to_string(),
    };
    match IndexerConnectionPool::connect(&config.to_string())
        .await
        .unwrap()
    {
        IndexerConnectionPool::Postgres(p) => p.acquire().await.unwrap(),
        _ => panic!("Expected Postgres connection."),
    }
}

pub fn test_sqlite_db_path() -> String {
    format!("sqlite://{}/test.db", WORKSPACE_ROOT)
}

pub async fn sqlite_connection() -> PoolConnection<Sqlite> {
    let db_url = test_sqlite_db_path();
    match IndexerConnectionPool::connect(&db_url).await.unwrap() {
        IndexerConnectionPool::Sqlite(p) => p.acquire().await.unwrap(),
        _ => panic!("Expected Sqlite connection."),
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

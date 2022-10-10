use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    future::Future,
    net::{SocketAddr, ToSocketAddrs},
};
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

const MAX_DATABASE_CONNECTION_ATTEMPTS: usize = 5;
const INITIAL_RETRY_DELAY_SECS: u64 = 2;

pub fn sha256_digest(blob: &Vec<u8>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(blob.as_slice());
    format!("{:x}", hasher.finalize())
}

pub fn trim_opt_env_key(key: &str) -> &str {
    // Abmiguous key: $FOO, non-ambiguous key: ${FOO}
    let not_ambiguous = key.starts_with("${");
    match not_ambiguous {
        false => &key[1..],
        true => &key[2..key.len() - 1],
    }
}

pub fn is_opt_env_var(key: &str) -> bool {
    key.starts_with('$') || (key.starts_with("${") && key.ends_with('}'))
}

pub fn derive_socket_addr(host: &String, port: &String) -> Result<SocketAddr> {
    let host = format!("{}:{}", host, port);
    match &host.parse() {
        Ok(sock) => Ok(*sock),
        Err(e) => {
            warn!(
                    "Failed to parse '{}' as a SocketAddr due to '{}'. Retrying using ToSocketAddrs.",
                    host, e
                );

            let mut addrs: Vec<_> = host
                .to_socket_addrs()
                .unwrap_or_else(|_| panic!("Unable to resolve domain for '{}'", host))
                .collect();

            let addr = addrs
                .pop()
                .unwrap_or_else(|| panic!("Could not derive SocketAddr from '{}'", host));

            info!("Parsed SocketAddr '{}' from '{}'", addr.to_string(), host);

            Ok(addr)
        }
    }
}

/// Attempt to connect to a database, retrying a number of times if a connection
/// can't be made. This function takes a closure with a database connection
/// function as an argument; said function should return a future that
/// resolves to a final value of type Result<T, sqlx::Error>.
pub async fn attempt_database_connection<F, Fut, T, U>(mut fut: F) -> T
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, U>>,
    U: std::error::Error,
{
    let mut remaining_retries = MAX_DATABASE_CONNECTION_ATTEMPTS;
    let mut delay = INITIAL_RETRY_DELAY_SECS;
    loop {
        match fut().await {
            Ok(t) => break t,
            Err(_) => {
                if remaining_retries > 0 {
                    warn!(
                            "Could not connect to database backend, retrying in {} seconds...",
                            delay
                        );
                    remaining_retries -= 1;
                    sleep(Duration::from_secs(delay)).await;
                    delay *= 2;
                } else {
                    panic!(
                        "Retry attempts exceeded; could not connect to database backend!"
                    )
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServiceStatus {
    OK,
    NotOk,
}

impl From<FuelNodeHealthResponse> for ServiceStatus {
    fn from(r: FuelNodeHealthResponse) -> Self {
        match r.up {
            true => ServiceStatus::OK,
            _ => ServiceStatus::NotOk,
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct FuelNodeHealthResponse {
    up: bool,
}

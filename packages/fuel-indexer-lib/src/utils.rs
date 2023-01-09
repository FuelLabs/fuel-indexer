use crate::defaults;
use anyhow::Result;
use fuel_indexer_types::Bytes32;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::str::FromStr;
use std::{
    fs::canonicalize,
    future::Future,
    net::{SocketAddr, ToSocketAddrs},
    path::Path,
};
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

const ROOT_DIRECTORY_NAME: &str = "fuel-indexer";

// Testing assets use relative paths, while production assets will use absolute paths
//
// If we can successfully find the local project root, then we're in the repository,
// and should prefix all relative asset paths with the project root. If we can't find
// the project root, then it's assumed we're not in a local repository, thus no prefix.
pub fn local_repository_root() -> Option<String> {
    let curr_filepath = canonicalize(file!()).unwrap();
    let mut curr_dir = Path::new(&curr_filepath);

    // 4 = -> src (1) -> fuel-indexer-lib (2) -> packages -> (3) -> fuel-indexer (4)
    let mut depth = 4;
    while depth > 0 {
        match curr_dir.parent() {
            Some(p) => {
                curr_dir = p;
                depth -= 1;
            }
            None => {
                return None;
            }
        }
    }

    if !curr_dir.is_dir() || curr_dir.file_name().unwrap() != ROOT_DIRECTORY_NAME {
        return None;
    }

    let root_dir = curr_dir.as_os_str().to_str().unwrap().to_string();

    Some(root_dir)
}

#[derive(Debug)]
pub struct AssetReloadRequest {
    pub namespace: String,
    pub identifier: String,
}

#[derive(Debug)]
pub struct IndexStopRequest {
    pub namespace: String,
    pub identifier: String,
}

#[derive(Debug)]
pub enum ServiceRequest {
    AssetReload(AssetReloadRequest),
    IndexStop(IndexStopRequest),
}

pub fn sha256_digest<T: AsRef<[u8]>>(blob: &T) -> String {
    let mut hasher = Sha256::new();
    hasher.update(blob);
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

pub fn derive_socket_addr(host: &String, port: &String) -> SocketAddr {
    let host = format!("{}:{}", host, port);
    SocketAddr::from_str(&host).unwrap_or_else(|e| {
            warn!(
                "Failed to parse '{}' as a SocketAddr due to '{}'. Retrying using ToSocketAddrs.",
                host, e
            );

            let mut addrs: Vec<_> = host
                .to_socket_addrs()
                .expect("Unable to resolve domain.")
                .collect();

            let addr = addrs.pop().expect("Could not derive SocketAddr from '{}'");

            info!("Parsed SocketAddr '{:?}' from '{}'", addr, host);

            addr
        })
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
    let mut remaining_retries = defaults::MAX_DATABASE_CONNECTION_ATTEMPTS;
    let mut delay = defaults::INITIAL_RETRY_DELAY_SECS;
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

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct FuelNodeHealthResponse {
    up: bool,
}

pub mod index_utils {
    use fuel_indexer_types::SizedAsciiString;

    use super::{sha256_digest, Bytes32};

    pub fn u64_id(d: &[u8; 8]) -> u64 {
        u64::from_le_bytes(*d)
    }

    pub fn first8_bytes_to_u64(data: impl AsRef<[u8]>) -> u64 {
        let data = sha256_digest(&data);
        let mut buff = [0u8; 8];
        buff.copy_from_slice(&data.as_bytes()[..8]);
        u64_id(&buff)
    }

    pub fn first32_bytes_to_bytes32(data: impl AsRef<[u8]>) -> Bytes32 {
        let data = sha256_digest(&data);
        let mut buff = [0u8; 32];
        buff.copy_from_slice(&data.as_bytes()[..32]);
        Bytes32::from(buff)
    }

    pub fn u64_id_from_inputs(id: &[u8; 32], inputs: Vec<u8>) -> u64 {
        let inputs = [id.to_vec(), inputs].concat();
        first8_bytes_to_u64(&inputs)
    }

    pub fn bytes32_from_inputs(id: &[u8; 32], inputs: Vec<u8>) -> Bytes32 {
        let inputs = [id.to_vec(), inputs].concat();
        first32_bytes_to_bytes32(&inputs)
    }

    pub fn trim_sized_ascii_string<const LEN: usize>(
        s: &SizedAsciiString<LEN>,
    ) -> String {
        let mut s = s.to_string();
        let n = s.trim_end_matches(' ').len();
        s.truncate(n);
        s
    }
}

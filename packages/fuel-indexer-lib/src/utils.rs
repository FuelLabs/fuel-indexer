use crate::{config::IndexerConfig, defaults};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    env,
    fs::canonicalize,
    future::Future,
    net::{SocketAddr, ToSocketAddrs},
    path::Path,
    str::FromStr,
};
use tokio::time::{sleep, Duration};
use tracing::{debug, info, warn};
use tracing_subscriber::filter::EnvFilter;

const RUST_LOG: &str = "RUST_LOG";
const HUMAN_LOGGING: &str = "HUMAN_LOGGING";

const ROOT_DIRECTORY_NAME: &str = "fuel-indexer";

/// Serialize a generic byte array reference.
pub fn serialize(obj: &impl Serialize) -> Vec<u8> {
    bincode::serialize(obj).expect("Serialize failed")
}

/// Deserialize a generic byte array reference.
pub fn deserialize<'a, T: Deserialize<'a>>(bytes: &'a [u8]) -> Result<T, String> {
    match bincode::deserialize(bytes) {
        Ok(obj) => Ok(obj),
        Err(e) => Err(format!("Bincode serde error {e:?}")),
    }
}

// Testing assets use relative paths, while production assets will use absolute paths
//
// If we can successfully find the local project root, then we're in the repository,
// and should prefix all relative asset paths with the project root. If we can't find
// the project root, then it's assumed we're not in a local repository, thus no prefix.
//
// This is specifically required for the trybuild test suite.
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

/// Request to reload the specified indexer executor using this indexer's current
/// assets in the database.
///
/// Sent from API server to indexer service.
#[derive(Debug)]
pub struct ReloadRequest {
    pub namespace: String,
    pub identifier: String,
    pub remove_data: bool,
    pub replace_indexer: bool,
}

/// Request to remove the specified indexer executor from the indexer service.
///
/// Sent from API server to indexer service.
#[derive(Debug)]
pub struct StopRequest {
    pub namespace: String,
    pub identifier: String,
    pub notify: Option<futures::channel::oneshot::Sender<()>>,
}

/// A general request sent from the API server to the indexer service.
#[derive(Debug)]
pub enum ServiceRequest {
    Reload(ReloadRequest),
    Stop(StopRequest),
}

/// Returns the lower hex representation of a [`sha2::SHA256`] digest of the provided input.
pub fn sha256_digest<T: AsRef<[u8]>>(b: &T) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b);
    format!("{:x}", hasher.finalize())
}

/// Trim the leading '$' or '${' and trailing '}' from an environment variable.
pub fn trim_opt_env_key(key: &str) -> &str {
    // Abmiguous key: $FOO, non-ambiguous key: ${FOO}
    let not_ambiguous = key.starts_with("${");
    match not_ambiguous {
        false => &key[1..],
        true => &key[2..key.len() - 1],
    }
}

/// Determine whether a given key is an environment variable.
pub fn is_opt_env_var(k: &str) -> bool {
    k.starts_with('$') || (k.starts_with("${") && k.ends_with('}'))
}

/// Derive the [`std::net::SocketAddr`] from a given host and port, falling back
/// to a DNS lookup using [`std::net::ToSocketAddrs`] if the host is not a valid IP address.
pub fn derive_socket_addr(host: &str, port: &str) -> SocketAddr {
    let host = format!("{host}:{port}");
    match SocketAddr::from_str(&host) {
        Ok(v) => v,
        Err(e) => {
            debug!("Failed to parse '{host}': {e}. Retrying...");
            let mut addrs: Vec<_> = host
                .to_socket_addrs()
                .unwrap_or_else(|e| panic!("Unable to resolve domain: {e}"))
                .collect();

            let addr = addrs.pop().expect("Could not derive SocketAddr from '{}'");

            info!("Parsed SocketAddr '{addr:?}' from '{host}'");

            addr
        }
    }
}

/// Attempt to connect to a database, with retries.
///
/// This function takes a closure with a database connection
/// function as an argument; said function should return a future that
/// resolves to a final value of type `Result<T, sqlx::Error>`.
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
                        "Could not connect to database. Retrying in {delay} seconds...",
                    );
                    remaining_retries -= 1;
                    sleep(Duration::from_secs(delay)).await;
                    delay *= 2;
                } else {
                    panic!("Retry attempts exceeded. Could not connect to database!")
                }
            }
        }
    }
}

/// Denotes the status of a service for the service health check.
#[derive(Debug, Serialize, Deserialize)]
pub enum ServiceStatus {
    /// The service is healthy.
    OK,

    /// The service is not healthy.
    NotOk,
}

impl From<FuelClientHealthResponse> for ServiceStatus {
    fn from(r: FuelClientHealthResponse) -> Self {
        match r.up {
            true => ServiceStatus::OK,
            _ => ServiceStatus::NotOk,
        }
    }
}

/// Response from the Fuel client health check.
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct FuelClientHealthResponse {
    /// `true` if the client is available, `false` otherwise.
    up: bool,
}

/// Initialize the logging context for the indexer service.
pub async fn init_logging(config: &IndexerConfig) -> anyhow::Result<()> {
    let level = env::var_os(RUST_LOG)
        .map(|x| x.into_string().unwrap())
        .unwrap_or("info".to_string());

    // We manually suppress some of the more verbose crate logging.
    if !config.verbose {
        std::env::set_var(
            RUST_LOG,
            format!("{level},wasmer_compiler_cranelift=warn,regalloc=warn,cranelift_codegen=warn"),
        );
    }

    let filter = match env::var_os(RUST_LOG) {
        Some(_) => {
            EnvFilter::try_from_default_env().expect("Invalid `RUST_LOG` provided")
        }
        None => EnvFilter::new("info"),
    };

    let human_logging = env::var_os(HUMAN_LOGGING)
        .map(|s| {
            bool::from_str(s.to_str().unwrap())
                .expect("Expected `true` or `false` to be provided for `HUMAN_LOGGING`")
        })
        .unwrap_or(true);

    let sub = tracing_subscriber::fmt::Subscriber::builder()
        .with_writer(std::io::stderr)
        .with_env_filter(filter);

    if human_logging {
        sub.with_ansi(true)
            .with_level(true)
            .with_line_number(true)
            .init();
    } else {
        sub.with_ansi(false)
            .with_level(true)
            .with_line_number(true)
            .json()
            .init();
    }
    Ok(())
}

/// Format a SQL query for logging.
pub fn format_sql_query(s: String) -> String {
    s.replace('\n', " ")
}

pub mod utils {

    use anyhow::Result;
    use serde::{Deserialize, Serialize};
    use sha2::{Digest, Sha256};
    use std::net::{SocketAddr, ToSocketAddrs};
    use tracing::{info, warn};

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

                let addr = addrs.pop().unwrap_or_else(|| {
                    panic!("Could not derive SocketAddr from '{}'", host)
                });

                info!("Parsed SocketAddr '{}' from '{}'", addr.to_string(), host);

                Ok(addr)
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
}

pub mod defaults {

    pub const FUEL_NODE_HOST: &str = "127.0.0.1";
    pub const FUEL_NODE_PORT: &str = "4000";

    pub const GRAPHQL_API_HOST: &str = "127.0.0.1";
    pub const GRAPHQL_API_PORT: &str = "29987";

    pub const DATABASE: &str = "postgres";
    pub const POSTGRES_DATABASE: &str = "postgres";
    pub const POSTGRES_USER: &str = "postgres";
    pub const POSTGRES_HOST: &str = "127.0.0.1";
    pub const POSTGRES_PORT: &str = "5432";
    pub const POSTGRES_PASSWORD: &str = "";

    pub const SQLITE_DATABASE: &str = "sqlite.db";

    pub const GRAPHQL_API_RUN_MIGRATIONS: Option<bool> = None;
}

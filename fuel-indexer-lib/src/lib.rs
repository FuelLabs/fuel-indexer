pub mod utils {

    use crate::defaults;
    use anyhow::Result;
    use serde::{Deserialize, Serialize};
    use sha2::{Digest, Sha256};
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

        let mut depth = 3;
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
                        panic!("Retry attempts exceeded; could not connect to database backend!")
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

    pub const ASSET_REFRESH_CHANNEL_SIZE: usize = 100;

    pub const MAX_DATABASE_CONNECTION_ATTEMPTS: usize = 5;
    pub const INITIAL_RETRY_DELAY_SECS: u64 = 2;
}

pub mod manifest {
    use anyhow::Result;
    use serde::{Deserialize, Serialize};
    use std::path::Path;
    use std::{fs::File, io::Read};

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Manifest {
        pub namespace: String,
        pub abi: String,
        pub identifier: String,
        pub graphql_schema: String,
        pub module: Module,
        pub contract_id: Option<String>,
        pub start_block: Option<u64>,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    #[serde(rename_all = "lowercase")]
    pub enum Module {
        Wasm(String),
        Native(String),
    }

    impl Module {
        pub fn path(&self) -> String {
            match self {
                Self::Wasm(o) => o.clone(),
                Self::Native(o) => o.clone(),
            }
        }
    }

    impl Manifest {
        pub fn from_file(path: &Path) -> Result<Self> {
            let mut file = File::open(path).unwrap_or_else(|_| {
                panic!("Manifest at '{}' does not exist", path.display())
            });
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            let manifest: Manifest = serde_yaml::from_str(&contents)?;
            Ok(manifest)
        }

        pub fn to_bytes(&self) -> Vec<u8> {
            serde_yaml::to_string(&self)
                .expect("Failed converting manifest to bytes.")
                .as_bytes()
                .to_vec()
        }

        pub fn graphql_schema(&self) -> Result<String> {
            let mut file = File::open(&self.graphql_schema)?;
            let mut schema = String::new();
            file.read_to_string(&mut schema)?;

            Ok(schema)
        }

        pub fn uid(&self) -> String {
            format!("{}.{}", &self.namespace, &self.identifier)
        }

        pub fn is_native(&self) -> bool {
            match &self.module {
                Module::Native(_o) => true,
                Module::Wasm(_o) => false,
            }
        }
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Event {
        pub trigger: String,
        pub payload: String,
    }
}

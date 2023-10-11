use crate::{graphql::GraphQLSchema, ExecutionSource};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    str::FromStr,
};
use thiserror::Error;

/// Result type returned from Manifest operations.
type ManifestResult<T> = Result<T, ManifestError>;

/// Error type returned from Manifest operations.
#[derive(Error, Debug)]
pub enum ManifestError {
    #[error("Compiler error: {0:#?}")]
    YamlError(#[from] serde_yaml::Error),
    #[error("Native module bytes not supported.")]
    NativeModuleError,
    #[error("File IO error: {0} {1:?}.")]
    FileError(String, #[source] std::io::Error),
}

/// Specifies which type of module is used to create this indexer.
///
/// When using a `Wasm` module, the WASM binary at the given path
/// is read and those bytes are registered into a `WasmIndexerExecutor`.
/// `Native` modules on the other hand do not require a path, because
/// native indexers compile to binaries that can be executed without having
/// to read the bytes of some compiled module.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Module {
    Wasm(String),
    Native,
}

impl From<PathBuf> for Module {
    fn from(path: PathBuf) -> Self {
        Self::Wasm(path.to_str().unwrap().to_string())
    }
}

impl ToString for Module {
    fn to_string(&self) -> String {
        match self {
            Self::Wasm(o) => o.to_string(),
            Self::Native => "native".to_string(),
        }
    }
}

impl AsRef<Path> for Module {
    fn as_ref(&self) -> &Path {
        match self {
            Self::Wasm(o) => Path::new(o),
            Self::Native => {
                unimplemented!("Only WASM execution supports module path access.")
            }
        }
    }
}

/// Represents the indexer manifest file.
///
/// This manifest file is a simple YAML file that is read and passed
/// to the excecutor to which the indexer is registered. This manifest
/// specifies various properties of how the indexer should be run in
/// the indexer executor (e.g., Where should the indexing start?).
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Manifest {
    /// Namespace of indexer.
    namespace: String,

    /// Identifier of indexer.
    identifier: String,

    /// Filepath to Sway contract ABI.
    abi: Option<String>,

    /// URL to Fuel client.
    ///
    /// Only used if `--indexer-net-config` is specified in `IndexerArgs`.
    fuel_client: Option<String>,

    /// Filepath to this indexer's GraphQL schema.
    graphql_schema: String,

    /// Executor module.
    module: Module,

    /// Whether or not to record metrics for this indexer.
    metrics: Option<bool>,

    /// Set of contract IDs this indexer should subscribe to.
    #[serde(
        serialize_with = "ContractIds::serialize",
        deserialize_with = "ContractIds::deserialize"
    )]
    contract_id: ContractIds,

    /// Block at which indexer should start.
    start_block: Option<u32>,

    /// Block at which indexer should stop.
    end_block: Option<u32>,

    /// When set to true, the indexer will resume from the block height at which it last stopped.
    #[serde(default)]
    resumable: Option<bool>,
}

impl Manifest {
    /// Derive an indexer manifest via the YAML file at the specified path.
    pub fn from_file(path: impl AsRef<Path>) -> ManifestResult<Self> {
        let mut file = File::open(&path).map_err(|e| {
            ManifestError::FileError(path.as_ref().display().to_string(), e)
        })?;
        let mut content = String::new();
        file.read_to_string(&mut content).map_err(|e| {
            ManifestError::FileError(path.as_ref().display().to_string(), e)
        })?;
        Self::try_from(content.as_str())
    }

    /// Return the raw GraphQL schema string for an indexer manifest.
    pub fn graphql_schema_content(&self) -> ManifestResult<GraphQLSchema> {
        let mut file = File::open(&self.graphql_schema)
            .map_err(|err| ManifestError::FileError(self.graphql_schema.clone(), err))?;
        let mut schema = String::new();
        file.read_to_string(&mut schema)
            .map_err(|err| ManifestError::FileError(self.graphql_schema.clone(), err))?;
        Ok(GraphQLSchema::new(schema))
    }

    /// Derive the unique identifier for a manifest.
    pub fn uid(&self) -> String {
        format!("{}.{}", &self.namespace, &self.identifier)
    }

    /// Determine what type of execution source this indexer is using.
    pub fn execution_source(&self) -> ExecutionSource {
        match &self.module {
            Module::Native => ExecutionSource::Native,
            Module::Wasm(_o) => ExecutionSource::Wasm,
        }
    }

    /// Return the bytes of the compiled indexer WASM module.
    ///
    /// Note that as mentioned, because native execution does not compile
    /// to a module that can be uploaded (as WASM execution does), there is
    /// no way to read module bytes if native execution is specified.
    pub fn module_bytes(&self) -> ManifestResult<Vec<u8>> {
        match &self.module {
            Module::Wasm(p) => {
                let mut bytes = Vec::<u8>::new();
                let mut file = File::open(p)
                    .map_err(|err| ManifestError::FileError(p.clone(), err))?;
                file.read_to_end(&mut bytes)
                    .map_err(|err| ManifestError::FileError(p.clone(), err))?;

                Ok(bytes)
            }
            Module::Native => {
                unimplemented!("Native execution does not support this method.")
            }
        }
    }

    /// Write this manifest to a given path.
    pub fn write(&self, path: &PathBuf) -> ManifestResult<()> {
        let mut file = File::create(path).map_err(|err| {
            ManifestError::FileError(path.to_str().unwrap_or_default().to_string(), err)
        })?;
        let content: Vec<u8> = Self::into(self.clone());
        file.write_all(&content).map_err(|err| {
            ManifestError::FileError(path.to_str().unwrap_or_default().to_string(), err)
        })?;
        Ok(())
    }

    /// Set the start block for this indexer.
    pub fn set_start_block(&mut self, block: u32) {
        self.start_block = Some(block);
    }

    /// Set the executor module for this indexer.
    pub fn set_module(&mut self, module: Module) {
        self.module = module;
    }

    /// Set the end block for this indexer.
    pub fn set_end_block(&mut self, block: u32) {
        self.end_block = Some(block);
    }

    /// Set the GraphQL schema for this indexer.
    pub fn set_graphql_schema(&mut self, schema: String) {
        self.graphql_schema = schema;
    }

    /// Set the contract ABI for this indexer.
    pub fn set_abi(&mut self, abi: String) {
        self.abi = Some(abi);
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn set_namespace(&mut self, namespace: String) {
        self.namespace = namespace;
    }

    pub fn set_identifier(&mut self, identifier: String) {
        self.identifier = identifier;
    }

    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub fn graphql_schema(&self) -> &str {
        &self.graphql_schema
    }

    pub fn start_block(&self) -> Option<u32> {
        self.start_block
    }

    pub fn contract_id(&self) -> &ContractIds {
        &self.contract_id
    }

    pub fn abi(&self) -> Option<&str> {
        self.abi.as_deref()
    }

    pub fn fuel_client(&self) -> Option<&str> {
        self.fuel_client.as_deref()
    }

    pub fn module(&self) -> &Module {
        &self.module
    }

    pub fn end_block(&self) -> Option<u32> {
        self.end_block
    }

    pub fn resumable(&self) -> Option<bool> {
        self.resumable
    }
}

impl TryFrom<&str> for Manifest {
    type Error = ManifestError;

    fn try_from(val: &str) -> ManifestResult<Self> {
        let manifest: Manifest = serde_yaml::from_str(val)?;
        Ok(manifest)
    }
}

impl From<Manifest> for Vec<u8> {
    fn from(manifest: Manifest) -> Self {
        serde_yaml::to_vec(&manifest).unwrap()
    }
}

impl TryFrom<&Vec<u8>> for Manifest {
    type Error = ManifestError;

    fn try_from(val: &Vec<u8>) -> ManifestResult<Self> {
        let manifest: Manifest = serde_yaml::from_slice(val)?;
        Ok(manifest)
    }
}

/// Represents contract IDs in a `Manifest` struct.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum ContractIds {
    /// Single represents a single contract ID as an `Option<String>`.
    #[serde(alias = "single")]
    Single(Option<String>),

    /// Multiple represents a vector of contracts IDs as a Vec<String>.
    #[serde(alias = "multiple")]
    Multiple(Vec<String>),
}

impl ContractIds {
    fn serialize<S>(ids: &ContractIds, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match ids {
            ContractIds::Single(Some(id)) => id.clone(),
            ContractIds::Multiple(ids) => {
                serde_json::to_string(ids).map_err(serde::ser::Error::custom)?
            }
            _ => return serializer.serialize_none(),
        };
        serializer.serialize_str(&s)
    }

    fn deserialize<'de, D>(deserializer: D) -> Result<ContractIds, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_yaml::Value::deserialize(deserializer)?;
        match value {
            serde_yaml::Value::String(s) => Ok(ContractIds::Single(Some(s))),
            serde_yaml::Value::Sequence(seq) => {
                let ids = seq
                    .into_iter()
                    .filter_map(|val| match val {
                        serde_yaml::Value::String(s) => Some(s),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                Ok(ContractIds::Multiple(ids.into_iter().collect()))
            }
            serde_yaml::Value::Null => Ok(ContractIds::Single(None)),
            _ => Err(serde::de::Error::custom("Invalid contract_id value")),
        }
    }
}

impl FromStr for ContractIds {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('[') {
            serde_json::from_str::<Vec<String>>(s)
                .map(ContractIds::Multiple)
                .map_err(|err| err.to_string())
        } else {
            Ok(ContractIds::Single(Some(s.to_string())))
        }
    }
}

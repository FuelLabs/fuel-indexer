use crate::graphql::GraphQLSchema;
use anyhow::Result;
use inflections::case::to_pascal_case;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
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
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Module {
    Wasm(String),
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
        }
    }
}

impl AsRef<Path> for Module {
    fn as_ref(&self) -> &Path {
        match self {
            Self::Wasm(o) => Path::new(o),
        }
    }
}

/// Predicates used by this indexer.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Predicates {
    /// Predicate templates
    templates: Option<Vec<PredicateTemplate>>,
}

impl Predicates {
    /// Get the predicate templates.
    pub fn templates(&self) -> Option<&[PredicateTemplate]> {
        self.templates.as_deref()
    }

    /// Check if this predicate set is empty.
    pub fn is_empty(&self) -> bool {
        self.templates.is_none()
    }
}

/// Represents a predicate template.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PredicateTemplate {
    /// Name of predicate.
    name: String,

    /// Hash of predicate bytecode used to uniquely identify predicate.
    id: String,

    /// Filepath to Sway predicate ABI.
    abi: String,
}

impl PredicateTemplate {
    /// Get the predicate name.
    pub fn name(&self) -> String {
        to_pascal_case(&self.name)
    }

    /// Get the predicate ID.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the predicate ABI.
    pub fn abi(&self) -> &str {
        &self.abi
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Contract {
    /// Filepath to Sway contract ABI.
    abi: Option<String>,

    /// Set of contract IDs this indexer should subscribe to.
    subsriptions: Option<Vec<String>>,
}

impl Contract {
    /// Get the contract ABI.
    pub fn abi(&self) -> Option<&str> {
        self.abi.as_deref()
    }

    /// Get the contract subscriptions.
    pub fn subscriptions(&self) -> Option<&[String]> {
        self.subsriptions.as_deref()
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

    /// Contract configuration.
    contract: Option<Contract>,

    /// URL to Fuel client.
    ///
    /// Only used if `--indexer-net-config` is specified in `IndexerArgs`.
    fuel_client: Option<String>,

    /// Filepath to this indexer's GraphQL schema.
    schema: String,

    /// Executor module.
    module: Module,

    /// Block at which indexer should start.
    start_block: Option<u32>,

    /// Block at which indexer should stop.
    end_block: Option<u32>,

    /// When set to true, the indexer will resume from the block height at which it last stopped.
    resumable: Option<bool>,

    /// Set of predicates used by this indexer.
    predicates: Option<Predicates>,
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
    pub fn schema_content(&self) -> ManifestResult<GraphQLSchema> {
        let mut file = File::open(&self.schema)
            .map_err(|err| ManifestError::FileError(self.schema.clone(), err))?;
        let mut schema = String::new();
        file.read_to_string(&mut schema)
            .map_err(|err| ManifestError::FileError(self.schema.clone(), err))?;
        Ok(GraphQLSchema::new(schema))
    }

    /// Derive the unique identifier for a manifest.
    pub fn uid(&self) -> String {
        format!("{}.{}", &self.namespace, &self.identifier)
    }

    /// Return the bytes of the compiled indexer WASM module.
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

    /// Get the indexer namespace.
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

    /// Get the indexer GraphQL schema.
    pub fn schema(&self) -> &str {
        &self.schema
    }

    /// Get the indexer start block.
    pub fn start_block(&self) -> Option<u32> {
        self.start_block
    }

    /// Get the indexer contract configuration.
    pub fn contract_abi(&self) -> Option<&str> {
        self.contract.as_ref().and_then(|c| c.abi())
    }

    /// Get the indexer contract subscriptions.
    pub fn contract_subscriptions(&self) -> Option<&[String]> {
        self.contract.as_ref().and_then(|c| c.subscriptions())
    }

    /// Get the indexer predicates.
    pub fn predicates(&self) -> Option<&Predicates> {
        self.predicates.as_ref()
    }

    /// Get the indexer Fuel client.
    pub fn fuel_client(&self) -> Option<&str> {
        self.fuel_client.as_deref()
    }

    /// Get the indexer module.
    pub fn module(&self) -> &Module {
        &self.module
    }

    /// Get the indexer end block.
    pub fn end_block(&self) -> Option<u32> {
        self.end_block
    }

    /// Get the indexer's resumability.
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

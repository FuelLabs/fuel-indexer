use anyhow::Result;
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
    #[error("File IO error: {0:?}.")]
    FileError(#[from] std::io::Error),
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

impl ToString for Module {
    fn to_string(&self) -> String {
        match self {
            Self::Wasm(o) => o.clone(),
            Self::Native => {
                unimplemented!("Only wasm execution supports module path access.")
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
    pub namespace: String,
    pub abi: Option<String>,
    pub identifier: String,
    pub fuel_client: Option<String>,
    pub graphql_schema: String,
    pub module: Module,
    pub metrics: Option<bool>,
    pub contract_id: Option<String>,
    pub start_block: Option<u64>,
    #[serde(default)]
    pub resumable: Option<bool>,
}

impl Manifest {
    /// Derive an indexer manifest via the YAML file at the specified path.
    pub fn from_file(path: impl AsRef<Path>) -> ManifestResult<Self> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Self::from_str(&content)
    }

    pub fn from_slice(s: &[u8]) -> ManifestResult<Self> {
        Ok(serde_yaml::from_slice(s)?)
    }

    pub fn to_bytes(&self) -> ManifestResult<Vec<u8>> {
        Ok(serde_yaml::to_string(&self)?.as_bytes().to_vec())
    }

    /// Return the raw GraphQL schema string for an indexer manifest.
    pub fn graphql_schema(&self) -> ManifestResult<String> {
        let mut file = File::open(&self.graphql_schema)?;
        let mut schema = String::new();
        file.read_to_string(&mut schema)?;
        Ok(schema)
    }

    /// Derive the unique identifier for a manifest.
    pub fn uid(&self) -> String {
        format!("{}.{}", &self.namespace, &self.identifier)
    }

    /// Determine whether this manifest supports native execution.
    pub fn is_native(&self) -> bool {
        match &self.module {
            Module::Native => true,
            Module::Wasm(_o) => false,
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
                let mut file = File::open(p)?;
                file.read_to_end(&mut bytes)?;

                Ok(bytes)
            }
            Module::Native => {
                unimplemented!("Native execution does not support this method.")
            }
        }
    }

    /// Write this manifest to a given path.
    pub fn write(&self, path: &PathBuf) -> ManifestResult<()> {
        let mut file = File::create(path)?;
        let content: Vec<u8> = Self::try_into(self.clone())?;
        file.write_all(&content)?;
        Ok(())
    }
}

impl TryFrom<&str> for Manifest {
    type Error = ManifestError;

    fn try_from(val: &str) -> ManifestResult<Self> {
        let manifest: Manifest = serde_yaml::from_str(val)?;
        Ok(manifest)
    }
}

impl TryInto<Vec<u8>> for Manifest {
    type Error = ManifestError;

    fn try_into(self) -> ManifestResult<Vec<u8>> {
        Ok(serde_yaml::to_string(&self)?.as_bytes().to_vec())
    }
}

impl TryFrom<&Vec<u8>> for Manifest {
    type Error = ManifestError;

    fn try_from(val: &Vec<u8>) -> ManifestResult<Self> {
        let manifest: Manifest = serde_yaml::from_slice(val)?;
        Ok(manifest)
    }
}

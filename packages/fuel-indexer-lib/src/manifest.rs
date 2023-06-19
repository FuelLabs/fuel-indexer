use crate::ExecutionSource;
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
    pub namespace: String,
    pub abi: Option<String>,
    pub identifier: String,
    pub fuel_client: Option<String>,
    pub graphql_schema: String,
    pub module: Module,
    pub metrics: Option<bool>,
    #[serde(
        serialize_with = "ContractIds::serialize",
        deserialize_with = "ContractIds::deserialize"
    )]
    pub contract_id: ContractIds,
    pub start_block: Option<u32>,
    pub end_block: Option<u32>,
    #[serde(default)]
    pub resumable: Option<bool>,
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
    pub fn graphql_schema(&self) -> ManifestResult<String> {
        let mut file = File::open(&self.graphql_schema)
            .map_err(|err| ManifestError::FileError(self.graphql_schema.clone(), err))?;
        let mut schema = String::new();
        file.read_to_string(&mut schema)
            .map_err(|err| ManifestError::FileError(self.graphql_schema.clone(), err))?;
        Ok(schema)
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
        let content: Vec<u8> = Self::try_into(self.clone())?;
        file.write_all(&content).map_err(|err| {
            ManifestError::FileError(path.to_str().unwrap_or_default().to_string(), err)
        })?;
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

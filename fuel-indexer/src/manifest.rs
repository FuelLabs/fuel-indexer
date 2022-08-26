use crate::handler::ReceiptEvent;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::{fs::File, io::Read};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Manifest {
    pub namespace: String,
    pub graphql_schema: PathBuf,
    pub wasm_module: Option<PathBuf>,
    pub start_block: Option<u64>,
    pub handlers: Vec<Handle>,
    pub test_events: Option<Vec<Event>>,
}

impl Manifest {
    pub fn new(namespace: String, graphql_schema: PathBuf, start_block: Option<u64>) -> Self {
        Self {
            namespace,
            graphql_schema,
            wasm_module: None,
            start_block,
            handlers: Vec::new(),
            test_events: None,
        }
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        let mut file = File::open(&path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let manifest: Manifest = serde_yaml::from_str(&contents)?;
        Ok(manifest)
    }

    pub fn from_file_as_bytes(path: &Path) -> Result<Vec<u8>> {
        let mut file = File::open(&path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents.as_bytes().to_vec())
    }

    pub fn load_schema_from_file(&self) -> Result<String> {
        let mut file = File::open(&self.graphql_schema)?;
        let mut schema = String::new();
        file.read_to_string(&mut schema)?;

        Ok(schema)
    }

    pub fn load_wasm_from_file(&self) -> Result<Vec<u8>> {
        let mut bytes = Vec::<u8>::new();
        if let Some(module) = &self.wasm_module {
            let mut file = File::open(module)?;

            file.read_to_end(&mut bytes)?;
        }
        Ok(bytes)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let contents = serde_yaml::to_string(self).unwrap();
        Ok(contents.as_bytes().to_vec())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Event {
    pub trigger: String,
    pub payload: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Handle {
    pub event: ReceiptEvent,
    pub handler: String,
}

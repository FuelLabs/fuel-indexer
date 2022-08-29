use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::{fs::File, io::Read};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Manifest {
    pub namespace: String,
    pub graphql_schema: String,
    pub wasm_module: Option<String>,
    pub start_block: Option<u64>,
    pub test_events: Option<Vec<Event>>,
}

impl Manifest {
    pub fn new(namespace: String, graphql_schema: String, start_block: Option<u64>) -> Self {
        Self {
            namespace,
            graphql_schema,
            wasm_module: None,
            start_block,
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

    pub fn graphql_schema(&self) -> Result<String> {
        let mut file = File::open(&self.graphql_schema)?;
        let mut schema = String::new();
        file.read_to_string(&mut schema)?;

        Ok(schema)
    }

    pub fn wasm_module(&self) -> Result<Vec<u8>> {
        let mut bytes = Vec::<u8>::new();
        if let Some(module) = &self.wasm_module {
            let mut file = File::open(module)?;

            file.read_to_end(&mut bytes)?;
        }
        Ok(bytes)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Event {
    pub trigger: String,
    pub payload: String,
}

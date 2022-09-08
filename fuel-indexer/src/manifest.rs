use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::{fs::File, io::Read};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Manifest {
    pub namespace: String,
    pub graphql_schema: String,
    pub module: Module,
    pub start_block: Option<u64>,
    pub test_events: Option<Vec<Event>>,
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
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Event {
    pub trigger: String,
    pub payload: String,
}

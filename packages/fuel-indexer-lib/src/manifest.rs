use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::{fs::File, io::Read};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Manifest {
    pub namespace: String,
    pub abi: Option<String>,
    pub identifier: String,
    pub graphql_schema: String,
    pub module: Module,
    pub metrics: Option<bool>,
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

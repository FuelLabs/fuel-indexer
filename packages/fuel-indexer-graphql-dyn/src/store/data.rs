use super::data_type::*;
use super::self_prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Data {
    Json(serde_json::Value),
}

impl Data {
    pub fn r#type(&self) -> &DataTypeId {
        todo!()
    }

    pub fn value(&self) -> &serde_json::Value {
        match self {
            Self::Json(json) => json,
        }
    }
}

impl From<serde_json::Value> for Data {
    fn from(value: serde_json::Value) -> Self {
        Self::Json(value)
    }
}

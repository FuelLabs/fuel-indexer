use super::meta::*;
use super::self_prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataTypeId {
    Unit,
    Bool,
    U8,
    U16,
    U32,
    U64,
    B256,
    Byte,
    Bytes,
    String,
    Array(Box<DataTypeId>),
    Name(Name),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DataType {
    Unit,
    Bool,
    U8,
    U16,
    U32,
    U64,
    B256,
    Byte,
    Bytes,
    String,
    Array(Box<DataTypeId>),
    Composite(Name, IndexMap<DataFieldId, DataFieldType>),
    Enum(Name, IndexMap<DataFieldId, DataFieldType>),
}

pub type DataFieldId = String;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataFieldType {
    pub name: Name,
    pub type_id: DataTypeId,
}

impl DataType {
    pub fn name(&self) -> Name {
        match self {
            Self::Unit => Name::new_pascal("Unit"),
            Self::Bool => Name::new_pascal("Boolean"),
            Self::U8 => Name::new_pascal("U8"),
            Self::U16 => Name::new_pascal("U16"),
            Self::U32 => Name::new_pascal("U32"),
            Self::U64 => Name::new_pascal("U64"),
            Self::B256 => Name::new_pascal("B256"),
            Self::Byte => Name::new_pascal("Byte"),
            Self::Bytes => Name::new_pascal("Bytes"),
            Self::String => Name::new_pascal("String"),
            Self::Array(ty) => Name::new_pascal(format!("[{}]", ty)),
            Self::Composite(name, _) => name.clone(),
            Self::Enum(name, _) => name.clone(),
        }
    }
}

impl FromStr for DataTypeId {
    type Err = anyhow::Error;
    fn from_str(str: &str) -> anyhow::Result<Self, Self::Err> {
        match str {
            "Unit" => Ok(Self::Unit),
            "Boolean" => Ok(Self::Bool),
            "U8" => Ok(Self::U8),
            "U16" => Ok(Self::U16),
            "U32" => Ok(Self::U32),
            "U64" => Ok(Self::U64),
            "B256" => Ok(Self::B256),
            "Byte" => Ok(Self::Byte),
            "Bytes" => Ok(Self::Bytes),
            "String" => Ok(Self::String),
            str if str.starts_with("Array__") => {
                todo!()
            }
            str => {
                let name = str.to_string();
                Ok(Self::Name(Name::new_pascal(name)))
            }
        }
    }
}

impl std::fmt::Display for DataTypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unit => write!(f, "Unit"),
            Self::Bool => write!(f, "Boolean"),
            Self::U8 => write!(f, "U8"),
            Self::U16 => write!(f, "U16"),
            Self::U32 => write!(f, "U32"),
            Self::U64 => write!(f, "U64"),
            Self::B256 => write!(f, "B256"),
            Self::Byte => write!(f, "Byte"),
            Self::Bytes => write!(f, "Bytes"),
            Self::String => write!(f, "String"),
            Self::Array(ty) => write!(f, "[{}]", ty),
            Self::Name(id) => write!(f, "{}", id),
        }
    }
}

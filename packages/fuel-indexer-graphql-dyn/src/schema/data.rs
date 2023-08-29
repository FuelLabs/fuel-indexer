use super::self_prelude::*;

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum DynDataTypeId {
    // GraphQL types
    ID,
    String,
    Int,
    Float,
    Boolean,
    List(Box<DynDataTypeId>),
    // Option(Box<DynDataTypeId>),
    // Fuel types
    Unit,
    U8,
    U16,
    U32,
    U64,
    B256,
    Bytes,
    Name(Name),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DynDataType {
    // GraphQL types
    ID,
    String,
    Int,
    Float,
    Boolean,
    List(Box<DynDataType>),
    // Option(Box<DynDataTypeId>),
    // Fuel types
    Unit,
    U8,
    U16,
    U32,
    U64,
    B256,
    Bytes,
    Object(Name, IndexMap<DynDataFieldId, DynDataFieldType>),
    Enum(Name, IndexMap<DynDataFieldId, DynDataFieldType>),
}

pub type DynDataFieldId = String;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DynDataFieldType {
    pub name: Name,
    pub data_type_id: DynDataTypeId,
    pub store_data_field_id: store::DataFieldId,
}

impl From<store::DataTypeId> for DynDataTypeId {
    fn from(store_data_type_id: store::DataTypeId) -> Self {
        match store_data_type_id {
            store::DataTypeId::Unit => Self::Unit,
            store::DataTypeId::Bool => Self::Boolean,
            store::DataTypeId::U8 => Self::U8,
            store::DataTypeId::U16 => Self::U16,
            store::DataTypeId::U32 => Self::U32,
            store::DataTypeId::U64 => Self::U64,
            store::DataTypeId::B256 => Self::B256,
            store::DataTypeId::Byte => Self::U8,
            store::DataTypeId::Bytes => Self::Bytes,
            store::DataTypeId::String => Self::String,
            store::DataTypeId::Array(ty) => Self::List(Box::new((*ty).into())),
            store::DataTypeId::Name(name) => Self::Name(name),
        }
    }
}

impl FromStr for DynDataTypeId {
    type Err = ();
    fn from_str(str: &str) -> Result<Self, Self::Err> {
        match str {
            // GraphQL types
            "ID" => Ok(Self::ID),
            "String" => Ok(Self::String),
            "Int" => Ok(Self::Int),
            "Float" => Ok(Self::Float),
            "Boolean" => Ok(Self::Boolean),
            // List(Box<DynDataTypeId>),
            // Option(Box<DynDataTypeId>),
            // Fuel types
            "Unit" => Ok(Self::Unit),
            "U8" => Ok(Self::U8),
            "U16" => Ok(Self::U16),
            "U32" => Ok(Self::U32),
            "U64" => Ok(Self::U64),
            "B256" => Ok(Self::B256),
            "Bytes" => Ok(Self::Bytes),
            str => Ok(Self::Name(Name::new_pascal(str))),
        }
    }
}

impl std::fmt::Display for DynDataTypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ID => write!(f, "ID"),
            Self::Int => write!(f, "Int"),
            Self::Float => write!(f, "Float"),
            Self::List(ty) => write!(f, "[{}]", ty),
            // Self::Option(ty) => write!(f, "{}", ty),
            Self::Unit => write!(f, "Unit"),
            Self::Boolean => write!(f, "Boolean"),
            Self::U8 => write!(f, "U8"),
            Self::U16 => write!(f, "U16"),
            Self::U32 => write!(f, "U32"),
            Self::U64 => write!(f, "U64"),
            Self::B256 => write!(f, "B256"),
            Self::Bytes => write!(f, "Bytes"),
            Self::String => write!(f, "String"),
            Self::Name(str) => write!(f, "{}", str),
        }
    }
}

impl DynDataType {
    pub fn name(&self) -> Name {
        match self {
            Self::ID => Name::new_pascal("ID"),
            Self::Int => Name::new_pascal("Int"),
            Self::Float => Name::new_pascal("Float"),
            Self::List(ty) => Name::new_pascal(format!("[{}]", ty.name())),
            // Self::Option(ty) => Name::new_pascal(&format!("{}!", ty)),
            Self::Unit => Name::new_pascal("Unit"),
            Self::Boolean => Name::new_pascal("Boolean"),
            Self::U8 => Name::new_pascal("U8"),
            Self::U16 => Name::new_pascal("U16"),
            Self::U32 => Name::new_pascal("U32"),
            Self::U64 => Name::new_pascal("U64"),
            Self::B256 => Name::new_pascal("B256"),
            Self::Bytes => Name::new_pascal("Bytes"),
            Self::String => Name::new_pascal("String"),
            Self::Object(name, _) => Name::new_pascal(format!("{}", name)),
            Self::Enum(name, _) => Name::new_pascal(format!("{}", name)),
        }
    }
}

impl DynDataFieldType {
    pub fn data_type_id(&self) -> &DynDataTypeId {
        &self.data_type_id
    }
    pub fn store_id(&self) -> &store::DataFieldId {
        &self.store_data_field_id
    }
}

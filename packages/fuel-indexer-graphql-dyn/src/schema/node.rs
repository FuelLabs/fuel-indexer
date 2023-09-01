use super::data::*;
use super::edge::*;
use super::self_prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DynNodeTypeId {
    pub store_id: store::ObjTypeId,
}

pub type DynNodeId = store::ObjId;

pub type DynNodeData = store::Obj;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DynNodeType {
    pub name: Name,
    pub fields: IndexMap<DynDataFieldId, DynNodeFieldType>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DynNodeFieldType {
    Data(DynDataFieldType),
    Ref(DynNodeRefFieldType),
    Connection(DynNodeConnectionFieldType),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DynNodeConnectionFieldType {
    pub name: Name,
    pub edge_type_id: DynEdgeTypeId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DynNodeRefFieldType {
    pub name: Name,
    pub store_id: store::DataFieldId,
    pub ref_node_type_id: DynNodeTypeId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DynNode(DynNodeId, DynNodeData);

impl From<DynNodeTypeId> for store::ObjTypeId {
    fn from(node_type_id: DynNodeTypeId) -> Self {
        node_type_id.store_id
    }
}

impl From<store::ObjTypeId> for DynNodeTypeId {
    fn from(store_type_id: store::ObjTypeId) -> Self {
        Self {
            store_id: store_type_id,
        }
    }
}

impl FromStr for DynNodeTypeId {
    type Err = ();
    fn from_str(str: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            store_id: store::ObjTypeId::from_str(str).map_err(|_| ())?,
        })
    }
}

impl fmt::Display for DynNodeTypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.store_id)
    }
}

impl DynNodeType {
    pub fn fields(&self) -> &IndexMap<DynDataFieldId, DynNodeFieldType> {
        &self.fields
    }
    pub fn field(&self, field_id: &DynDataFieldId) -> Option<&DynNodeFieldType> {
        self.fields.get(field_id)
    }
}

impl DynNodeFieldType {
    pub fn name(&self) -> Name {
        match self {
            Self::Data(data) => data.name.clone(),
            Self::Ref(data) => data.name.clone(),
            Self::Connection(data) => data.name.clone(),
        }
    }

    pub fn as_data_field(&self) -> &DynDataFieldType {
        match self {
            Self::Data(data) => data,
            _ => panic!("DynNodeFieldType::as_data: not a data field"),
        }
    }
    pub fn as_ref_field(&self) -> &DynNodeRefFieldType {
        match self {
            Self::Ref(data) => data,
            _ => panic!("DynNodeFieldType::as_ref: not a ref field"),
        }
    }
    pub fn as_connection_field(&self) -> &DynNodeConnectionFieldType {
        match self {
            Self::Connection(data) => data,
            _ => panic!("DynNodeFieldType::as_connection: not a connection field"),
        }
    }
}

impl DynNodeConnectionFieldType {
    pub fn edge_type_id(&self) -> &DynEdgeTypeId {
        &self.edge_type_id
    }
}

impl DynNodeRefFieldType {
    pub fn store_id(&self) -> &DynDataFieldId {
        &self.store_id
    }
    pub fn ref_node_type_id(&self) -> &DynNodeTypeId {
        &self.ref_node_type_id
    }
}

impl DynNode {
    pub fn new(id: impl Into<DynNodeId>, data: DynNodeData) -> Self {
        Self(id.into(), data)
    }

    pub fn id(&self) -> &DynNodeId {
        &self.0
    }
    pub fn type_id(&self) -> DynNodeTypeId {
        self.1.type_id().clone().into()
    }
    pub fn data(&self) -> &serde_json::Value {
        self.1.data().value()
    }
}

use super::data::*;
use super::node::*;
use super::self_prelude::*;

pub type DynEdgeTypeId = store::AssocTypeId;
pub type DynEdgeData = store::Assoc;
pub type DynEdgeTime = store::AssocTime;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct DynEdgeId(DynEdgeTypeId, DynNodeId, DynNodeId);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DynEdgeType {
    pub name: Name,
    pub tail: DynNodeTypeId,
    pub head: DynNodeTypeId,
    pub fields: IndexMap<DynDataFieldId, DynDataFieldType>,
}

#[derive(Clone)]
pub struct DynEdge(DynEdgeTypeId, DynNodeId, DynEdgeData);

impl DynEdgeId {
    pub fn new(
        type_id: impl Into<DynEdgeTypeId>,
        tail_id: impl Into<DynNodeId>,
        head_id: impl Into<DynNodeId>,
    ) -> Self {
        Self(type_id.into(), tail_id.into(), head_id.into())
    }
}

impl DynEdgeType {
    pub fn tail_type_id(&self) -> &DynNodeTypeId {
        &self.tail
    }
    pub fn head_type_id(&self) -> &DynNodeTypeId {
        &self.head
    }
    pub fn fields(&self) -> &IndexMap<DynDataFieldId, DynDataFieldType> {
        &self.fields
    }
    pub fn field(&self, field_id: &DynDataFieldId) -> Option<&DynDataFieldType> {
        self.fields.get(field_id)
    }
}

impl DynEdge {
    pub fn new(
        type_id: impl Into<DynEdgeTypeId>,
        tail_id: impl Into<DynNodeId>,
        data: impl Into<DynEdgeData>,
    ) -> Self {
        Self(type_id.into(), tail_id.into(), data.into())
    }

    pub fn type_id(&self) -> &DynEdgeTypeId {
        &self.0
    }
    pub fn tail_id(&self) -> &DynNodeId {
        &self.1
    }
    pub fn head_id(&self) -> &DynNodeId {
        self.2.id()
    }
    pub fn time(&self) -> &DynEdgeTime {
        self.2.time()
    }
    pub fn data(&self) -> &serde_json::Value {
        self.2.data().value()
    }
}

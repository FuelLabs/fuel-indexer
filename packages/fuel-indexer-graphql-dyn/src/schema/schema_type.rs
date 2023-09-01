use super::data::*;
use super::edge::*;
use super::node::*;
use super::self_prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DynSchemaType {
    pub data: IndexMap<DynDataTypeId, DynDataType>,
    pub node: IndexMap<DynNodeTypeId, DynNodeType>,
    pub edge: IndexMap<DynEdgeTypeId, DynEdgeType>,
}

impl DynSchemaType {
    pub fn data(&self, id: &DynDataTypeId) -> &DynDataType {
        match id {
            DynDataTypeId::ID => Some(&DynDataType::ID),
            DynDataTypeId::Name(name) => self.data.values().find(|data_type| {
                dbg!(&data_type.name(), name);
                data_type.name() == *name
            }),
            _ => self.data.get(id),
        }
        .unwrap_or_else(|| {
            panic!("DynamicSchemaType::data_type: data type {:?} not found", id)
        })
    }
    pub fn node(&self, id: &DynNodeTypeId) -> &DynNodeType {
        self.node.get(id).unwrap_or_else(|| {
            panic!("DynamicSchemaType::node_type: node type {:?} not found", id)
        })
    }
    pub fn node_field(
        &self,
        id: &DynNodeTypeId,
        field_id: &DynDataFieldId,
    ) -> &DynNodeFieldType {
        self.node(id).fields.get(field_id).unwrap_or_else(|| {
            panic!(
                "DynamicSchemaType::node_field: field {:?} not found",
                field_id
            )
        })
    }
    pub fn edge(&self, id: &DynEdgeTypeId) -> &DynEdgeType {
        self.edge.get(id).unwrap_or_else(|| {
            panic!("DynamicSchemaType::edge_type: edge type {:?} not found", id)
        })
    }
    pub fn edge_field(
        &self,
        id: &DynEdgeTypeId,
        field_id: &DynDataFieldId,
    ) -> &DynDataFieldType {
        self.edge(id).fields.get(field_id).unwrap_or_else(|| {
            panic!(
                "DynamicSchemaType::edge_field: field {:?} not found",
                field_id
            )
        })
    }
}

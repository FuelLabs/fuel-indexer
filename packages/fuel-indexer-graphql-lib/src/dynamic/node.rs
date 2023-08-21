//! `async_graphql::dynamic` extensions for handling GraphQL nodes.
//! See: https://graphql.org/learn/global-object-identification/#node-interface

use super::data::*;
use super::prelude::*;
use super::resolver::*;

pub type DynamicNodeLocalId = String;

pub type DynamicNodeTypeId = String;

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct DynamicNodeId(DynamicNodeTypeId, DynamicNodeLocalId);
impl DynamicNodeId {
    pub fn new(
        type_id: impl Into<DynamicNodeTypeId>,
        local_id: impl Into<DynamicNodeLocalId>,
    ) -> Self {
        Self(type_id.into(), local_id.into())
    }
}
impl DynamicNodeId {
    pub fn type_id(&self) -> &DynamicNodeTypeId {
        &self.0
    }
    pub fn local_id(&self) -> &DynamicNodeLocalId {
        &self.1
    }
}
impl FromStr for DynamicNodeId {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.splitn(2, ':');
        let type_id = split.next().unwrap().parse().unwrap();
        let local_id = split.next().unwrap().to_string();
        Ok(Self(type_id, local_id))
    }
}
impl From<DynamicNodeId> for String {
    fn from(id: DynamicNodeId) -> String {
        format!("{}:{}", id.type_id(), id.local_id())
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct DynamicNode(DynamicNodeTypeId, DynamicNodeLocalId, DynamicData);
impl DynamicNode {
    pub fn new(
        type_id: impl Into<DynamicNodeTypeId>,
        local_id: impl Into<DynamicNodeLocalId>,
        data: impl Into<DynamicData>,
    ) -> Self {
        Self(type_id.into(), local_id.into(), data.into())
    }

    pub fn id(&self) -> DynamicNodeId {
        DynamicNodeId::new(self.type_id(), self.local_id())
    }
    pub fn type_id(&self) -> &DynamicNodeTypeId {
        &self.0
    }
    pub fn local_id(&self) -> &DynamicNodeLocalId {
        &self.1
    }
    pub fn data(&self) -> &DynamicData {
        &self.2
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct DynamicConnectionField {
    pub id: DynamicFieldId,
    pub ref_node_type_id: DynamicNodeTypeId,
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct DynamicRefField {
    pub id: DynamicFieldId,
    pub data_field_id: DynamicFieldId,
    pub ref_node_type_id: DynamicNodeTypeId,
}

#[derive(Clone, PartialEq, Eq)]
pub struct DynamicNodeType {
    pub id: DynamicNodeTypeId,
    pub data_fields: Vec<DynamicDataField>,
    pub ref_fields: Vec<DynamicRefField>,
    pub connection_fields: Vec<DynamicConnectionField>,
}

impl From<DynamicNodeType> for Object {
    fn from(value: DynamicNodeType) -> Self {
        let name = TypeRef::node(value.id.clone());

        let mut object = <Object as NodeObject<DynamicResolver>>::new_node(&name);

        // Add data fields
        for data_field in &value.data_fields {
            object = <Object as NodeObject<DynamicResolver>>::data_field(
                object,
                data_field.id(),
                TypeRef::named_nn(data_field.data_type().to_string()),
            );
        }

        // Add ref fields
        for ref_field in &value.ref_fields {
            object = <Object as NodeObject<DynamicResolver>>::ref_field(
                object,
                ref_field.id.clone(),
                TypeRef::named_nn(ref_field.ref_node_type_id.to_string()),
            );
        }

        // Add connection fields
        for connection_field in &value.connection_fields {
            object = <Object as NodeObject<DynamicResolver>>::connection_field(
                object,
                connection_field.id.clone(),
                TypeRef::named_nn(TypeRef::connection(
                    connection_field.ref_node_type_id.clone(),
                )),
            );
        }

        object
    }
}

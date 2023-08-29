//! `async_graphql::dynamic` extensions for handling GraphQL connections.
//! See: https://graphql.org/learn/pagination/#end-of-list-counts-and-connections
//! See: https://relay.dev/graphql/connections.htm#sec-Connection-Types

use super::node::*;
use super::prelude::*;

pub type DynamicEdgeTypeId = String;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct DynamicEdgeId(DynamicEdgeTypeId, DynamicNodeLocalId, DynamicNodeLocalId);
impl DynamicEdgeId {
    pub fn new(
        type_id: impl Into<DynamicEdgeTypeId>,
        tail_local_id: impl Into<DynamicNodeLocalId>,
        head_local_id: impl Into<DynamicNodeLocalId>,
    ) -> Self {
        Self(type_id.into(), tail_local_id.into(), head_local_id.into())
    }
}
impl From<DynamicEdgeId> for String {
    fn from(id: DynamicEdgeId) -> String {
        format!("{}:{}:{}", id.0, id.1, id.2)
    }
}

pub type DynamicEdgeData = serde_json::Value;

#[derive(Clone, PartialEq, Eq)]
pub struct DynamicEdge(
    DynamicEdgeTypeId,
    DynamicNodeLocalId,
    DynamicNodeLocalId,
    DynamicEdgeData,
);
impl DynamicEdge {
    pub fn new(
        id: impl Into<DynamicEdgeTypeId>,
        tail_local_id: impl Into<DynamicNodeLocalId>,
        head_local_id: impl Into<DynamicNodeLocalId>,
        data: impl Into<DynamicEdgeData>,
    ) -> Self {
        Self(
            id.into(),
            tail_local_id.into(),
            head_local_id.into(),
            data.into(),
        )
    }

    pub fn type_id(&self) -> &DynamicEdgeTypeId {
        &self.0
    }
    pub fn tail_local_id(&self) -> &DynamicNodeLocalId {
        &self.1
    }
    pub fn head_local_id(&self) -> &DynamicNodeLocalId {
        &self.2
    }
    pub fn data(&self) -> &DynamicEdgeData {
        &self.3
    }
}

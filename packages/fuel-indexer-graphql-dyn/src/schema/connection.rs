use super::node::*;
use super::paging::*;
use super::self_prelude::*;

#[derive(Clone, PartialEq, Eq)]
pub struct DynConnectionEdge {
    pub node_id: DynNodeId,
    pub cursor: Cursor,
}

#[derive(Clone, PartialEq, Eq)]
pub struct DynConnection {
    pub total_count: usize,
    pub edges: Vec<DynConnectionEdge>,
    pub page_info: DynPageInfo,
}

impl DynConnectionEdge {
    pub fn new(
        node_id: &(impl Into<DynNodeId> + Clone),
        cursor: &(impl Into<Cursor> + Clone),
    ) -> Self {
        Self {
            node_id: node_id.clone().into(),
            cursor: cursor.clone().into(),
        }
    }

    pub fn node_id(&self) -> &DynNodeId {
        &self.node_id
    }
    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }
}

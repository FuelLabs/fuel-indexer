//! `async_graphql::dynamic` extensions for handling GraphQL connections.
//! See: https://graphql.org/learn/pagination/#end-of-list-counts-and-connections
//! See: https://relay.dev/graphql/connections.htm#sec-Connection-Types

use super::node::*;
use super::paging::*;

#[derive(Clone, PartialEq, Eq)]
pub struct DynamicConnectionEdge {
    pub node_id: DynamicNodeId,
    pub cursor: DynamicCursor,
}

#[derive(Clone, PartialEq, Eq)]
pub struct DynamicConnection {
    pub total_count: usize,
    pub edges: Vec<DynamicConnectionEdge>,
    pub page_info: DynamicPageInfo,
}

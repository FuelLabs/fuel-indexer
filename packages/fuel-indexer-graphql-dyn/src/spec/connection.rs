//! `async_graphql::dynamic` extensions for handling GraphQL connections.
//! See: https://graphql.org/learn/pagination/#end-of-list-counts-and-connections
//! See: https://relay.dev/graphql/connections.htm#sec-Connection-Types

use super::node::*;
use super::paging::*;
use super::self_prelude::*;

#[extension_trait]
pub impl ConnectionTypeRef for TypeRef {
    fn connection(node_name: impl Into<String>) -> String {
        format!("{}Connection", node_name.into())
    }
    fn connection_edge(node_name: impl Into<String>) -> String {
        format!("{}Edge", node_name.into())
    }
}

#[extension_trait]
pub impl ConnectionObject for Object {
    fn new_connection<Resolver: ConnectionResolver>(
        edge_name: impl Into<String>,
        node_name: impl Into<String>,
    ) -> Self {
        let edge_name = edge_name.into();
        let node_name = node_name.into();
        Self::new(TypeRef::connection(&edge_name))
            .field(Field::new(
                "totalCount",
                TypeRef::named_nn(TypeRef::INT),
                Resolver::resolve_total_count,
            ))
            .field(Field::new(
                "nodes",
                TypeRef::named_nn_list_nn(node_name),
                Resolver::resolve_nodes,
            ))
            .field(Field::new(
                "edges",
                TypeRef::named_nn_list_nn(TypeRef::connection_edge(&edge_name)),
                Resolver::resolve_edges,
            ))
            .field(Field::new(
                "pageInfo",
                TypeRef::named_nn(TypeRef::PAGE_INFO),
                Resolver::resolve_page_info,
            ))
    }
}

pub trait ConnectionResolver: Send + Sync + 'static {
    fn resolve_total_count(ctx: ResolverContext) -> FieldFuture;
    fn resolve_nodes(ctx: ResolverContext) -> FieldFuture;
    fn resolve_edges(ctx: ResolverContext) -> FieldFuture;
    fn resolve_page_info(ctx: ResolverContext) -> FieldFuture;
}

#[extension_trait]
pub impl ConnectionEdgeObject for Object {
    fn new_connection_edge<Resolver: ConnectionEdgeResolver>(
        name: impl Into<String>,
        head_name: impl Into<String>,
    ) -> Self {
        Self::new(TypeRef::connection_edge(name))
            .field(Field::new(
                "node",
                TypeRef::named_nn(TypeRef::node(head_name)),
                Resolver::resolve_node,
            ))
            .field(Field::new(
                "cursor",
                TypeRef::named_nn(TypeRef::CURSOR),
                Resolver::resolve_cursor,
            ))
    }
}

pub trait ConnectionEdgeResolver: Send + Sync + 'static {
    fn resolve_node(ctx: ResolverContext) -> FieldFuture;
    fn resolve_cursor(ctx: ResolverContext) -> FieldFuture;
}

#[extension_trait]
pub impl ConnectionField for Field {
    /// Add connection arguments to a field.
    /// See: https://relay.dev/graphql/connections.htm#sec-Arguments
    fn connection_arguments(self) -> Self {
        self.pagination_arguments()
    }
}

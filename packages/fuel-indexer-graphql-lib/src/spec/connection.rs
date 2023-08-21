//! `async_graphql::dynamic` extensions for handling GraphQL connections.
//! See: https://graphql.org/learn/pagination/#end-of-list-counts-and-connections
//! See: https://relay.dev/graphql/connections.htm#sec-Connection-Types

use super::node::*;
use super::paging::*;
use super::prelude::*;

#[extension_trait]
pub impl ConnectionTypeRef for TypeRef {
    fn edge(node_name: impl Into<String>) -> String {
        format!("{}Edge", node_name.into())
    }
    fn connection(node_name: impl Into<String>) -> String {
        format!("{}Connection", node_name.into())
    }
}

#[extension_trait]
pub impl<Resolver: ConnectionResolver> ConnectionObject<Resolver> for Object {
    fn new_connection(node_name: impl Into<String>) -> Self {
        let node_name = node_name.into();
        Self::new(TypeRef::connection(node_name.clone()))
            .field(Field::new(
                "totalCount",
                TypeRef::named_nn(TypeRef::INT),
                Resolver::resolve_total_count,
            ))
            .field(Field::new(
                "nodes",
                TypeRef::named_nn_list_nn(node_name.clone()),
                Resolver::resolve_nodes,
            ))
            .field(Field::new(
                "edges",
                TypeRef::named_nn_list_nn(TypeRef::edge(node_name)),
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
pub impl<Resolver: EdgeResolver> EdgeObject<Resolver> for Object {
    fn new_edge(name: impl Into<String>, head_name: impl Into<String>) -> Self {
        Self::new(TypeRef::edge(name))
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

pub trait EdgeResolver: Send + Sync + 'static {
    fn resolve_node(ctx: ResolverContext) -> FieldFuture;
    fn resolve_cursor(ctx: ResolverContext) -> FieldFuture;
}

#[extension_trait]
pub impl FieldConnectionExt for Field {
    /// Add connection arguments to a field.
    /// See: https://relay.dev/graphql/connections.htm#sec-Arguments
    fn connection_arguments(self) -> Self {
        // Forward pagination arguments
        self.argument(
            InputValue::new("first", TypeRef::named(TypeRef::INT)).description(
                "Paginate forward, returning the given amount of edges at most.",
            ),
        )
        .argument(
            InputValue::new("after", TypeRef::named(TypeRef::CURSOR))
                .description("Return edges after the given cursor."),
        )
        // Backward pagination arguments
        .argument(
            InputValue::new("last", TypeRef::named(TypeRef::INT)).description(
                "Paginate backward, returning the given amount of edges at most.",
            ),
        )
        .argument(
            InputValue::new("before", TypeRef::named(TypeRef::CURSOR))
                .description("Return edges before the given cursor."),
        )
    }
}

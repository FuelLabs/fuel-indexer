//! `async_graphql::dynamic` extensions for handling GraphQL connections.
//! See: https://graphql.org/learn/pagination/#end-of-list-counts-and-connections
//! See: https://relay.dev/graphql/connections.htm#sec-Connection-Types

use super::node::*;
use super::self_prelude::*;

#[extension_trait]
pub impl QueryTypeRef for TypeRef {
    const QUERY: &'static str = "Query";
}

#[extension_trait]
pub impl QueryObject for Object {
    fn new_query<Resolver: QueryResolver>() -> Self {
        Self::new(TypeRef::QUERY)
            .node_by_id_field::<Resolver>()
            .node_by_ids_field::<Resolver>()
    }

    /// See: https://graphql.org/learn/global-object-identification/#node-root-field
    fn node_by_id_field<Resolver: QueryResolver>(self) -> Self {
        self.field(
            Field::new(
                "node",
                TypeRef::named(TypeRef::NODE),
                Resolver::resolve_node_by_id,
            )
            .node_by_id_arguments(),
        )
    }
    /// See: https://graphql.org/learn/global-object-identification/#plural-identifying-root-fields
    fn node_by_ids_field<Resolver: QueryResolver>(self) -> Self {
        self.field(
            Field::new(
                "nodes",
                TypeRef::named_list_nn(TypeRef::NODE),
                Resolver::resolve_nodes_by_id,
            )
            .node_by_ids_arguments(),
        )
    }
}

pub trait QueryResolver: Send + Sync + 'static {
    fn resolve_node_by_id(ctx: ResolverContext) -> FieldFuture;
    fn resolve_nodes_by_id(ctx: ResolverContext) -> FieldFuture;
}

#[extension_trait]
pub impl QueryField for Field {
    /// Add singular identifying arguments to a root field.
    /// See: https://graphql.org/learn/global-object-identification/#node-root-field
    fn node_by_id_arguments(self) -> Self {
        self.argument(
            InputValue::new("id", TypeRef::named_nn(TypeRef::ID))
                .description("ID of the node."),
        )
    }

    /// Add plural identifying arguments to a root field.
    /// See: https://graphql.org/learn/global-object-identification/#plural-identifying-root-fields
    fn node_by_ids_arguments(self) -> Self {
        self.argument(
            InputValue::new("ids", TypeRef::named_nn_list_nn(TypeRef::ID))
                .description("IDs of the nodes."),
        )
    }
}

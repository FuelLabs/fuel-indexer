//! `async_graphql::dynamic` extensions for handling GraphQL edges.
//! See: https://graphql.org/learn/pagination/#pagination-and-edges
//! See: https://relay.dev/graphql/connections.htm#sec-Edge-Types.Fields

pub use super::node::*;
use async_graphql::dynamic::{Field, Object, SchemaBuilder, TypeRef};
use extension_trait::extension_trait;

#[extension_trait]
pub impl TypeRefEdgeExt for TypeRef {
    fn edge(node_name: impl Into<String>) -> String {
        format!("{}Edge", node_name.into())
    }
}

#[extension_trait]
pub impl ObjectEdgeExt for Object {
    fn new_edge(node_name: impl Into<String>) -> Self {
        let node_name = node_name.into();
        Self::new(TypeRef::edge(node_name.clone()))
            .field(Field::new(
                "node",
                TypeRef::named_nn(node_name),
                |_| unimplemented!(),
            ))
            .field(Field::new(
                "cursor",
                TypeRef::named_nn(TypeRef::STRING),
                |_| unimplemented!(),
            ))
    }
}

#[extension_trait]
pub impl SchemaBuilderEdgeExt for SchemaBuilder {
    fn register_edge_types(self) -> Self {
        self.register_node_types()
    }
}

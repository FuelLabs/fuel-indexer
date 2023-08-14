//! `async_graphql::dynamic` extensions for handling GraphQL connections.
//! See: https://graphql.org/learn/pagination/#end-of-list-counts-and-connections
//! See: https://relay.dev/graphql/connections.htm#sec-Connection-Types

pub use super::{edge::*, filtering::*, node::*, ordering::*, paging::*};
use async_graphql::dynamic::{
    Field, InputObject, InputValue, Object, SchemaBuilder, TypeRef,
};
use extension_trait::extension_trait;

#[extension_trait]
pub impl TypeRefConnectionExt for TypeRef {
    fn connection(node_name: impl Into<String>) -> String {
        format!("{}Connection", node_name.into())
    }
}

#[extension_trait]
pub impl ObjectConnectionExt for Object {
    fn new_connection(node_name: impl Into<String>) -> Self {
        let node_name = node_name.into();
        Self::new(TypeRef::connection(node_name.clone()))
            .field(Field::new(
                "totalCount",
                TypeRef::named_nn(TypeRef::INT),
                |_| unimplemented!(),
            ))
            .field(Field::new(
                "nodes",
                TypeRef::named_nn_list_nn(node_name.clone()),
                |_| unimplemented!(),
            ))
            .field(Field::new(
                "edges",
                TypeRef::named_nn_list_nn(TypeRef::edge(node_name)),
                |_| unimplemented!(),
            ))
            .field(Field::new(
                "pageInfo",
                TypeRef::named_nn(TypeRef::PAGE_INFO),
                |_| unimplemented!(),
            ))
    }
}

#[extension_trait]
pub impl FieldConnectionExt for Field {
    fn connection_arguments(self, node_name: impl Into<String>) -> Self {
        let node_name = node_name.into();
        self.filtering_arguments(node_name.clone())
            .ordering_arguments(node_name)
            .paging_arguments()
    }
}

#[extension_trait]
pub impl SchemaBuilderConnectionExt for SchemaBuilder {
    fn register_connection_types(self) -> Self {
        let id_filter_input = InputObject::new_eq_filter(TypeRef::ID);
        let node_filter_input = InputObject::new_filter(TypeRef::NODE).field(
            InputValue::new("id", TypeRef::named(TypeRef::filter_input(TypeRef::ID))),
        );
        let node_order_input = InputObject::new_order(TypeRef::NODE).field(
            InputValue::new("id", TypeRef::named(TypeRef::ORDER_DIRECTION)),
        );
        self.register_edge_types()
            .register_filtering_types()
            .register_ordering_types()
            .register_paging_types()
            .register(id_filter_input)
            .register(node_filter_input)
            .register(node_order_input)
    }
}

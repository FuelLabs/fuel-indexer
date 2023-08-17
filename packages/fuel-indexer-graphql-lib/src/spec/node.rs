//! `async_graphql::dynamic` extensions for handling GraphQL nodes.
//! See: https://graphql.org/learn/global-object-identification/#node-interface

use super::prelude::*;

#[extension_trait]
pub impl SchemaBuilderNodeExt for SchemaBuilder {
    fn register_node_types(self) -> Self {
        let node_interface = Interface::new_node(TypeRef::NODE);
        self.register(node_interface)
    }
}

#[extension_trait]
pub impl TypeRefNodeExt for TypeRef {
    const NODE: &'static str = "Node";
    fn node(node_name: impl Into<String>) -> String {
        node_name.into()
    }
}

#[extension_trait]
pub impl InterfaceNodeExt for Interface {
    fn new_node(name: impl Into<String>) -> Self {
        Self::new(name).node_fields()
    }

    fn node_fields(self) -> Self {
        self.field(InterfaceField::new("id", TypeRef::named_nn(TypeRef::ID)))
    }
}

#[extension_trait]
pub impl FieldNodeExt for Field {
    /// Add singular Node identifying arguments to a root field.
    /// See: https://graphql.org/learn/global-object-identification/#node-root-field
    fn node_singular_id_root_arguments(self) -> Self {
        self.argument(
            InputValue::new("id", TypeRef::named_nn(TypeRef::ID))
                .description("ID of the node."),
        )
    }

    /// Add plural Node identifying arguments to a root field.
    /// See: https://graphql.org/learn/global-object-identification/#plural-identifying-root-fields
    fn node_plural_id_root_arguments(self) -> Self {
        self.argument(
            InputValue::new("ids", TypeRef::named_nn_list_nn(TypeRef::ID))
                .description("IDs of the nodes."),
        )
    }
}

#[extension_trait]
pub impl ObjectNodeExt for Object {
    /// Add singular identifying Node field to a root Query object.
    /// See: https://graphql.org/learn/global-object-identification/#node-root-field
    fn node_singular_id_root_field<F>(
        self,
        name: impl Into<String>,
        node_name: impl Into<String>,
        resolver_fn: F,
    ) -> Self
    where
        F: for<'a> Fn(ResolverContext<'a>) -> FieldFuture<'a> + Send + Sync + 'static,
    {
        let node_name = node_name.into();
        self.field(
            Field::new(
                name,
                TypeRef::named_nn(TypeRef::node(node_name.clone())),
                resolver_fn,
            )
            .argument(
                InputValue::new("id", TypeRef::named_nn(TypeRef::ID))
                    .description(format!("ID of the {} node to return.", node_name)),
            ),
        )
    }

    /// Add plural identifying Node field to a root Query object.
    /// See: https://graphql.org/learn/global-object-identification/#plural-identifying-root-fields
    fn node_plural_id_root_field<F>(
        self,
        name: impl Into<String>,
        node_name: impl Into<String>,
        resolver_fn: F,
    ) -> Self
    where
        F: for<'a> Fn(ResolverContext<'a>) -> FieldFuture<'a> + Send + Sync + 'static,
    {
        let node_name = node_name.into();
        self.field(
            Field::new(
                name,
                TypeRef::named_list_nn(TypeRef::node(node_name.clone())),
                resolver_fn,
            )
            .argument(
                InputValue::new("ids", TypeRef::named_nn_list_nn(TypeRef::ID))
                    .description(format!("IDs of the {} nodes to return.", node_name)),
            ),
        )
    }
}

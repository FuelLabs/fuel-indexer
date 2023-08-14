//! `async_graphql::dynamic` extensions for handling GraphQL nodes.
//! See: https://graphql.org/learn/global-object-identification/#node-interface

use async_graphql::dynamic::{
    Field, Interface, InterfaceField, Object, SchemaBuilder, TypeRef,
};
use extension_trait::extension_trait;

#[extension_trait]
pub impl TypeRefNodeExt for TypeRef {
    const NODE: &'static str = "Node";
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
pub impl ObjectNodeExt for Object {
    fn new_node(name: impl Into<String>) -> Self {
        Self::new(name).implement(TypeRef::NODE).node_fields()
    }

    fn node_fields(self) -> Self {
        self.field(Field::new(
            "id",
            TypeRef::named_nn(TypeRef::ID),
            |_| unimplemented!(),
        ))
    }
}

#[extension_trait]
pub impl SchemaBuilderNodeExt for SchemaBuilder {
    fn register_node_types(self) -> Self {
        let node_interface = Interface::new_node(TypeRef::NODE);
        self.register(node_interface)
    }
}

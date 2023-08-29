//! `async_graphql::dynamic` extensions for handling GraphQL nodes.
//! See: https://graphql.org/learn/global-object-identification/#node-interface

use super::connection::*;
use super::prelude::*;

#[extension_trait]
pub impl NodeTypeRef for TypeRef {
    const NODE: &'static str = "Node";
    fn node(node_name: impl Into<String>) -> String {
        node_name.into()
    }
}

#[extension_trait]
pub impl NodeInterface for Interface {
    fn new_node() -> Self {
        Self::new(TypeRef::NODE)
            .field(InterfaceField::new("id", TypeRef::named_nn(TypeRef::ID)))
    }
}

#[extension_trait]
pub impl<Resolver: NodeResolver> NodeObject<Resolver> for Object {
    fn new_node(name: impl Into<String>) -> Self {
        Self::new(TypeRef::node(name))
            .implement(TypeRef::NODE)
            .field(Field::new(
                "id",
                TypeRef::named_nn(TypeRef::ID),
                Resolver::resolve_id,
            ))
    }

    fn data_field(self, name: impl Into<String>, ty: impl Into<TypeRef>) -> Self {
        self.field(Field::new(name, ty, Resolver::resolve_data))
    }
    fn ref_field(self, name: impl Into<String>, ty: impl Into<TypeRef>) -> Self {
        self.field(Field::new(name, ty, Resolver::resolve_ref))
    }
    fn connection_field(self, name: impl Into<String>, ty: impl Into<TypeRef>) -> Self {
        self.field(
            Field::new(name, ty, Resolver::resolve_connection).connection_arguments(),
        )
    }
}

pub trait NodeResolver: Send + Sync + 'static {
    fn resolve_id(ctx: ResolverContext) -> FieldFuture;
    fn resolve_data(ctx: ResolverContext) -> FieldFuture;
    fn resolve_ref(ctx: ResolverContext) -> FieldFuture;
    fn resolve_connection(ctx: ResolverContext) -> FieldFuture;
}

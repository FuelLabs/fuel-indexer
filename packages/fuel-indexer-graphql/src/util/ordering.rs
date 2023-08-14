//! `async_graphql::dynamic` extensions for handling connection ordering.

use async_graphql::dynamic::{
    Enum, Field, InputObject, InputValue, SchemaBuilder, TypeRef,
};
use extension_trait::extension_trait;

#[extension_trait]
pub impl TypeRefOrderingExt for TypeRef {
    const ORDER_DIRECTION: &'static str = "OrderDirection";

    fn order_input(value_name: impl Into<String>) -> String {
        format!("{}OrderInput", value_name.into())
    }
}

#[extension_trait]
pub impl InputObjectOrderingExt for InputObject {
    fn new_order(value_name: impl Into<String>) -> Self {
        let name = TypeRef::order_input(value_name);
        Self::new(name)
    }
}

#[extension_trait]
pub impl SchemaBuilderOrderingExt for SchemaBuilder {
    fn register_ordering_types(self) -> Self {
        let order_direction_enum =
            Enum::new(TypeRef::ORDER_DIRECTION).item("asc").item("desc");
        self.register(order_direction_enum)
    }
}

#[extension_trait]
pub impl FieldOrderingExt for Field {
    /// Add ordering arguments to a field.
    fn ordering_arguments(self, value_name: impl Into<String>) -> Self {
        let order_name = TypeRef::order_input(value_name);
        self.argument(InputValue::new("order", TypeRef::named(order_name)))
    }
}

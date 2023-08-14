//! `async_graphql::dynamic` extensions for handling connection filtering.

use async_graphql::dynamic::{Field, InputObject, InputValue, SchemaBuilder, TypeRef};
use extension_trait::extension_trait;

#[extension_trait]
pub impl TypeRefFilteringExt for TypeRef {
    fn filter_input(value_name: impl Into<String>) -> String {
        format!("{}FilterInput", value_name.into())
    }
}

#[extension_trait]
pub impl InputObjectFilteringExt for InputObject {
    fn new_filter(value_name: impl Into<String>) -> Self {
        let name = TypeRef::filter_input(value_name);
        Self::new(name).filter_fields()
    }

    fn new_eq_filter(value_name: impl Into<String>) -> Self {
        let name = TypeRef::filter_input(value_name);
        Self::new(name.clone())
            .filter_fields()
            .filter_eq_fields(name)
    }

    fn new_ord_filter(value_name: impl Into<String>) -> Self {
        let name = TypeRef::filter_input(value_name);
        Self::new(name.clone())
            .filter_fields()
            .filter_eq_fields(name.clone())
            .filter_ord_fields(name)
    }

    fn filter_fields(self) -> Self {
        let name = self.type_name().to_string();
        self.field(InputValue::new("and", TypeRef::named_nn_list(name.clone())))
            .field(InputValue::new("or", TypeRef::named_nn_list(name.clone())))
            .field(InputValue::new("not", TypeRef::named_nn_list(name)))
    }

    fn filter_eq_fields(self, value_name: impl Into<String>) -> Self {
        let value_name = value_name.into();
        self.field(InputValue::new("eq", TypeRef::named(value_name.clone())))
            .field(InputValue::new("in", TypeRef::named_nn_list(value_name)))
    }

    fn filter_ord_fields(self, value_name: impl Into<String>) -> Self {
        let value_name = value_name.into();
        self.field(InputValue::new("gt", TypeRef::named(value_name.clone())))
            .field(InputValue::new("gte", TypeRef::named(value_name.clone())))
            .field(InputValue::new("lt", TypeRef::named(value_name.clone())))
            .field(InputValue::new("lte", TypeRef::named(value_name)))
    }
}

#[extension_trait]
pub impl SchemaBuilderFilteringExt for SchemaBuilder {
    fn register_filtering_types(self) -> Self {
        self
    }
}

#[extension_trait]
pub impl FieldFilteringExt for Field {
    fn filtering_arguments(self, value_name: impl Into<String>) -> Self {
        let filter_name = TypeRef::filter_input(value_name);
        self.argument(InputValue::new("filter", TypeRef::named(filter_name)))
    }
}

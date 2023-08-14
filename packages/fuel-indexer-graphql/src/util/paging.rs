//! `async_graphql::dynamic` extensions for handling pagination.
//! See: https://graphql.org/learn/pagination/
use async_graphql::dynamic::{Field, InputValue, Object, SchemaBuilder, TypeRef};
use extension_trait::extension_trait;

#[extension_trait]
pub impl TypeRefPagingExt for TypeRef {
    const PAGE_INFO: &'static str = "PageInfo";
}

#[extension_trait]
pub impl SchemaBuilderPagingExt for SchemaBuilder {
    fn register_paging_types(self) -> Self {
        // PageInfo
        // See: https://relay.dev/graphql/connections.htm#sec-PageInfo
        let page_info_object = Object::new(TypeRef::PAGE_INFO)
            .field(Field::new(
                "hasNextPage",
                TypeRef::named_nn(TypeRef::BOOLEAN),
                |_| unimplemented!(),
            ))
            .field(Field::new(
                "hasPreviousPage",
                TypeRef::named_nn(TypeRef::BOOLEAN),
                |_| unimplemented!(),
            ))
            .field(Field::new(
                "startCursor",
                TypeRef::named(TypeRef::STRING),
                |_| unimplemented!(),
            ))
            .field(Field::new(
                "endCursor",
                TypeRef::named(TypeRef::STRING),
                |_| unimplemented!(),
            ));
        self.register(page_info_object)
    }
}

#[extension_trait]
pub impl FieldPagingExt for Field {
    /// Add pagination arguments to a field.
    /// See: https://relay.dev/graphql/connections.htm#sec-Arguments
    fn paging_arguments(self) -> Self {
        // Forward pagination arguments
        self.argument(
            InputValue::new("first", TypeRef::named(TypeRef::INT)).description(
                "Paginate forward, returning the given amount of edges at most.",
            ),
        )
        .argument(
            InputValue::new("after", TypeRef::named(TypeRef::STRING))
                .description("Return edges after the given cursor."),
        )
        // Backward pagination arguments
        .argument(
            InputValue::new("last", TypeRef::named(TypeRef::INT)).description(
                "Paginate backward, returning the given amount of edges at most.",
            ),
        )
        .argument(
            InputValue::new("before", TypeRef::named(TypeRef::STRING))
                .description("Return edges before the given cursor."),
        )
    }
}

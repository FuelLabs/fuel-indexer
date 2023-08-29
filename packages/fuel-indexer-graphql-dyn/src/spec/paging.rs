//! `async_graphql::dynamic` extensions for handling pagination.
//! See: https://graphql.org/learn/pagination/

use super::self_prelude::*;

pub type Cursor = String;

#[extension_trait]
pub impl PagingTypeRef for TypeRef {
    const CURSOR: &'static str = "String";
    const PAGE_INFO: &'static str = "PageInfo";
}

#[extension_trait]
pub impl PagingField for Field {
    fn pagination_arguments(self) -> Self {
        self.forward_pagination_arguments()
            .backward_pagination_arguments()
    }
    /// Add forward pagination arguments to a field.
    /// See: https://relay.dev/graphql/connections.htm#sec-Forward-pagination-arguments
    fn forward_pagination_arguments(self) -> Self {
        self.argument(
            InputValue::new("first", TypeRef::named(TypeRef::INT)).description(
                "Paginate forward, returning the given amount of edges at most.",
            ),
        )
        .argument(
            InputValue::new("after", TypeRef::named(TypeRef::CURSOR))
                .description("Return edges after the given cursor."),
        )
    }
    /// Add backward pagination arguments to a field.
    /// See: https://relay.dev/graphql/connections.htm#sec-Backward-pagination-arguments
    fn backward_pagination_arguments(self) -> Self {
        self.argument(
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

/// See: https://relay.dev/graphql/connections.htm#sec-PageInfo
#[extension_trait]
pub impl PageInfoObject for Object {
    fn new_page_info<Resolver: PageInfoResolver>() -> Self {
        Self::new(TypeRef::PAGE_INFO)
            .field(Field::new(
                "hasNextPage",
                TypeRef::named_nn(TypeRef::BOOLEAN),
                Resolver::resolve_has_next_page,
            ))
            .field(Field::new(
                "hasPreviousPage",
                TypeRef::named_nn(TypeRef::BOOLEAN),
                Resolver::resolve_has_previous_page,
            ))
            .field(Field::new(
                "startCursor",
                TypeRef::named(TypeRef::CURSOR),
                Resolver::resolve_start_cursor,
            ))
            .field(Field::new(
                "endCursor",
                TypeRef::named(TypeRef::CURSOR),
                Resolver::resolve_end_cursor,
            ))
    }
}

pub trait PageInfoResolver: Send + Sync + 'static {
    fn resolve_has_next_page(ctx: ResolverContext) -> FieldFuture;
    fn resolve_has_previous_page(ctx: ResolverContext) -> FieldFuture;
    fn resolve_start_cursor(ctx: ResolverContext) -> FieldFuture;
    fn resolve_end_cursor(ctx: ResolverContext) -> FieldFuture;
}

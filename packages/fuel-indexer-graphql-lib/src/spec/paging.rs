//! `async_graphql::dynamic` extensions for handling pagination.
//! See: https://graphql.org/learn/pagination/

use super::prelude::*;

pub type Cursor = String;

#[extension_trait]
pub impl PagingTypeRef for TypeRef {
    const CURSOR: &'static str = "String";
    const PAGE_INFO: &'static str = "PageInfo";
}

/// See: https://relay.dev/graphql/connections.htm#sec-PageInfo
#[extension_trait]
pub impl<Resolver: PageInfoResolver> PageInfoObject<Resolver> for Object {
    fn new_page_info() -> Self {
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

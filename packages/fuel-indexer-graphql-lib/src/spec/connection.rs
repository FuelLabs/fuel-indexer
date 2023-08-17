//! `async_graphql::dynamic` extensions for handling GraphQL connections.
//! See: https://graphql.org/learn/pagination/#end-of-list-counts-and-connections
//! See: https://relay.dev/graphql/connections.htm#sec-Connection-Types

use super::node::*;
use super::prelude::*;

#[extension_trait]
pub impl SchemaBuilderConnectionExt for SchemaBuilder {
    fn register_connection_types(self) -> Self {
        self.register_node_types()
    }
}

#[extension_trait]
pub impl TypeRefConnectionExt for TypeRef {
    const CURSOR: &'static str = "String";
    const PAGE_INFO: &'static str = "PageInfo";
    fn edge(node_name: impl Into<String>) -> String {
        format!("{}Edge", node_name.into())
    }
    fn connection(node_name: impl Into<String>) -> String {
        format!("{}Connection", node_name.into())
    }
}

#[extension_trait]
pub impl FieldConnectionExt for Field {
    /// Add connection arguments to a field.
    /// See: https://relay.dev/graphql/connections.htm#sec-Arguments
    fn pagination_arguments(self) -> Self {
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

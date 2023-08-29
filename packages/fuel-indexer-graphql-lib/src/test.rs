pub mod prelude {
    pub use super::util::*;
    pub use assert_matches::*;
    pub use async_graphql::dynamic::*;
    pub use async_graphql::{Request, Response, ServerError};
    pub use async_trait::async_trait;
    pub use extension_trait::extension_trait;
    pub use graphql_parser::*;
    pub use insta::*;
    pub use serde_json::{json, Value as JsonValue};
    pub use velcro::hash_map;
}

pub mod util {
    pub use super::prelude::*;

    pub async fn execute_query(
        schema: &Schema,
        query: impl Into<String>,
        root_value: Option<FieldValue<'static>>,
    ) -> Result<Response, Vec<ServerError>> {
        let request = Request::new(query);
        let request = if let Some(root_value) = root_value {
            request.root_value(root_value)
        } else {
            request.into()
        };
        let response = schema.execute(request).await;
        response.into_result()
    }

    #[extension_trait]
    pub impl TestSchema for Schema {
        fn pretty_sdl(&self) -> String {
            let sdl = self.sdl();
            let ugly = parse_schema::<&str>(&sdl).unwrap();
            ugly.format(&Style::default())
        }
    }
}

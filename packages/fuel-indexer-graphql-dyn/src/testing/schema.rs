use super::schema_type::*;
use super::self_prelude::*;
use super::store::*;
use crate::schema::*;

use async_graphql::dynamic::*;
use graphql_parser::{parse_schema, Style};

#[extension_trait]
pub impl TestSchema for Schema {
    fn build_test() -> DynSchemaBuilder {
        let store = new_test_store().unwrap();
        let schema_type = new_test_schema_type(&store.r#type).unwrap();
        DynSchemaBuilder::new(&schema_type, Arc::new(Mutex::new(store)))
    }

    fn pretty_sdl(&self) -> String {
        let sdl = self.sdl();
        let ugly = parse_schema::<&str>(&sdl).unwrap();
        ugly.format(&Style::default())
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use insta::assert_snapshot;

    use super::*;

    #[tokio::test]
    async fn sdl() {
        // Build the schema
        let schema = Schema::build_test();
        let schema = schema.finish();
        assert_matches!(schema, Ok(_));
        let schema = schema.unwrap();

        // Print the schema
        let text = schema.pretty_sdl();
        assert_snapshot!(text);
    }
}

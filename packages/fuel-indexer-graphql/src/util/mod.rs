mod connection;
mod edge;
mod filtering;
mod graph;
mod node;
mod ordering;
mod paging;

pub use connection::*;
pub use edge::*;
pub use filtering::*;
pub use graph::*;
pub use node::*;
pub use ordering::*;
pub use paging::*;

#[cfg(test)]
pub(self) mod test {
    pub use assert_matches::*;
    pub use async_graphql::dynamic::*;
    pub use extension_trait::*;
    pub use graphql_parser::*;
    pub use insta::*;

    #[extension_trait]
    pub impl SchemaTestExt for Schema {
        fn test_build() -> SchemaBuilder {
            let mut schema = Schema::build("Query", None, None);

            // Insert a dummy query to avoid erroring from the lack of it.
            let query = Object::new("Query").field(Field::new(
                "dummy",
                TypeRef::named("Int"),
                |_| unimplemented!(),
            ));
            schema = schema.register(query);

            schema
        }

        fn pretty_sdl(&self) -> String {
            let sdl = self.sdl();
            let ugly = parse_schema::<&str>(&sdl).unwrap();
            ugly.format(&Style::default())
        }
    }
}

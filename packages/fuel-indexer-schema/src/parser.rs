use crate::{
    utils::{build_schema_fields_and_types_map, build_schema_types_set, BASE_SCHEMA},
    IndexerSchemaError, IndexerSchemaResult,
};
use async_graphql_parser::{parse_schema, types::ServiceDocument};
use std::collections::{HashMap, HashSet};

/// A wrapper object used to encapsulate a lot of the boilerplate logic related
/// to parsing schema, creating mappings of types, fields, objects, etc.
#[derive(Debug, Default)]
pub struct ParsedGraphQLSchema {
    /// Namespace of the indexer.
    pub namespace: String,

    /// Identifier of the indexer.
    pub identifier: String,

    /// Whether we're building schema for native execution.
    pub is_native: bool,

    /// All unique names of types in the schema (whether objects, enums, or scalars).
    pub type_names: HashSet<String>,

    /// All unique names of enums in the schema.
    pub enum_names: HashSet<String>,

    /// All unique names of types for which tables should _not_ be created.
    pub non_indexable_type_names: HashSet<String>,

    /// All unique names of types that have already been parsed.
    pub parsed_type_names: HashSet<String>,

    /// All unique names of types that are possible foreign keys.
    pub foreign_key_names: HashSet<String>,

    /// A mapping of fully qualitified field names to their field types.
    pub field_type_mappings: HashMap<String, String>,

    /// All unique names of scalar types in the schema.
    pub scalar_names: HashSet<String>,
}

impl ParsedGraphQLSchema {
    /// Create a new ParsedGraphQLSchema.
    pub fn new(
        namespace: &str,
        identifier: &str,
        is_native: bool,
        ast: &ServiceDocument,
    ) -> IndexerSchemaResult<Self> {
        let base_ast =
            parse_schema(BASE_SCHEMA).map_err(IndexerSchemaError::ParseError)?;

        let (mut type_names, _) = build_schema_types_set(ast);
        let (scalar_names, _) = build_schema_types_set(&base_ast);
        type_names.extend(scalar_names.clone());

        Ok(Self {
            namespace: namespace.to_string(),
            identifier: identifier.to_string(),
            is_native,
            type_names,
            enum_names: HashSet::new(),
            non_indexable_type_names: HashSet::new(),
            parsed_type_names: HashSet::new(),
            foreign_key_names: HashSet::new(),
            field_type_mappings: build_schema_fields_and_types_map(ast)?,
            scalar_names,
        })
    }

    /// Whether the schema has a scalar type with the given name.
    pub fn has_scalar(&self, name: &str) -> bool {
        self.scalar_names.contains(name)
    }

    /// Whether the given field type name is a possible foreign key.
    pub fn is_possible_foreign_key(&self, name: &str) -> bool {
        self.parsed_type_names.contains(name) && !self.has_scalar(name)
    }

    /// Whether the given field type name is a type from which tables are created.
    #[allow(unused)]
    pub fn is_non_indexable_type(&self, name: &str) -> bool {
        self.non_indexable_type_names.contains(name)
    }

    /// Whether the given field type name is an enum type.
    pub fn is_enum_type(&self, name: &str) -> bool {
        self.enum_names.contains(name)
    }
}

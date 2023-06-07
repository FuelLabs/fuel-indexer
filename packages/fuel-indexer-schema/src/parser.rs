use crate::{
    utils::{build_schema_fields_and_types_map, build_schema_types_set, BASE_SCHEMA},
    IndexerSchemaError, IndexerSchemaResult,
};
use async_graphql_parser::{parse_schema, types::ServiceDocument};
use std::collections::{HashMap, HashSet};

/// A wrapper object used to encapsulate a lot of the boilerplate logic related
/// to parsing schema, creating mappings of types, fields, objects, etc.
#[derive(Debug, Clone)]
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

    /// A mapping of fully qualified field names to their field types.
    pub field_type_mappings: HashMap<String, String>,

    /// All unique names of scalar types in the schema.
    pub scalar_names: HashSet<String>,

    /// The parsed schema.
    pub ast: ServiceDocument,

    /// Entities that contain list fields.
    pub entities_with_list_fields: HashMap<String, HashSet<String>>,
}

impl Default for ParsedGraphQLSchema {
    fn default() -> Self {
        let ast = parse_schema(BASE_SCHEMA)
            .map_err(IndexerSchemaError::ParseError)
            .expect("Bad schema");

        Self {
            namespace: "".to_string(),
            identifier: "".to_string(),
            is_native: false,
            type_names: HashSet::new(),
            enum_names: HashSet::new(),
            non_indexable_type_names: HashSet::new(),
            parsed_type_names: HashSet::new(),
            field_type_mappings: HashMap::new(),
            scalar_names: HashSet::new(),
            ast,
            entities_with_list_fields: HashMap::new(),
        }
    }
}

impl ParsedGraphQLSchema {
    /// Create a new ParsedGraphQLSchema.
    pub fn new(
        namespace: &str,
        identifier: &str,
        is_native: bool,
        schema: Option<&str>,
    ) -> IndexerSchemaResult<Self> {
        let mut ast =
            parse_schema(BASE_SCHEMA).map_err(IndexerSchemaError::ParseError)?;
        let mut type_names = HashSet::new();
        let (scalar_names, _) = build_schema_types_set(&ast);
        type_names.extend(scalar_names.clone());

        if let Some(schema) = schema {
            ast = parse_schema(schema).map_err(IndexerSchemaError::ParseError)?;
            let (other_type_names, _) = build_schema_types_set(&ast);
            type_names.extend(other_type_names);
        }

        Ok(Self {
            namespace: namespace.to_string(),
            identifier: identifier.to_string(),
            is_native,
            type_names,
            enum_names: HashSet::new(),
            non_indexable_type_names: HashSet::new(),
            parsed_type_names: HashSet::new(),
            field_type_mappings: build_schema_fields_and_types_map(&ast)?,
            scalar_names,
            ast,
            entities_with_list_fields: HashMap::new(),
        })
    }

    /// Whether the schema has a scalar type with the given name.
    pub fn has_scalar(&self, name: &str) -> bool {
        self.scalar_names.contains(name)
    }

    /// Whether the given field type name is a possible foreign key.
    pub fn is_possible_foreign_key(&self, name: &str) -> bool {
        self.parsed_type_names.contains(name)
            && !self.has_scalar(name)
            && !self.is_non_indexable_non_enum(name)
    }

    /// Whether the given field type name is a type from which tables are created.
    #[allow(unused)]
    pub fn is_non_indexable_non_enum(&self, name: &str) -> bool {
        self.non_indexable_type_names.contains(name) && !self.is_enum_type(name)
    }

    /// Whether the given field type name is an enum type.
    pub fn is_enum_type(&self, name: &str) -> bool {
        self.enum_names.contains(name)
    }

    /// Whether the field of a given entity is a list type.
    pub fn is_list_type(&self, entity_name: &str, field_name: &str) -> bool {
        if let Some(set) = self.entities_with_list_fields.get(entity_name) {
            set.contains(field_name)
        } else {
            false
        }
    }
}

use crate::{utils::*, IndexerSchemaError, IndexerSchemaResult};
use async_graphql_parser::{
    parse_schema,
    types::{ServiceDocument, TypeKind, TypeSystemDefinition},
};
use fuel_indexer_database_types::directives;
use std::collections::{BTreeMap, HashMap, HashSet};

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

    /// All unique names of union types in the schema.
    pub union_names: HashSet<String>,

    /// All objects and their field names and types, indexed by object name.
    pub object_field_mappings: HashMap<String, BTreeMap<String, String>>,

    /// All unique names of types for which tables should _not_ be created.
    pub virtual_type_names: HashSet<String>,

    /// All unique names of types that have already been parsed.
    pub parsed_type_names: HashSet<String>,

    /// A mapping of fully qualified field names to their field types.
    pub field_type_mappings: HashMap<String, String>,

    // A mapping of fully qualified field names to their respective optionalities.
    pub field_type_optionality: HashMap<String, bool>,

    /// All unique names of scalar types in the schema.
    pub scalar_names: HashSet<String>,

    /// The parsed schema.
    pub ast: ServiceDocument,
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
            union_names: HashSet::new(),
            virtual_type_names: HashSet::new(),
            parsed_type_names: HashSet::new(),
            field_type_mappings: HashMap::new(),
            object_field_mappings: HashMap::new(),
            field_type_optionality: HashMap::new(),
            scalar_names: HashSet::new(),
            ast,
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

        let mut object_field_mappings = HashMap::new();
        let mut parsed_type_names = HashSet::new();
        let mut enum_names = HashSet::new();
        let mut union_names = HashSet::new();
        let mut virtual_type_names = HashSet::new();
        let mut field_type_mappings = HashMap::new();
        let mut field_type_optionality = HashMap::new();

        // Parse _everything_ in the GraphQL schema
        if let Some(schema) = schema {
            ast = parse_schema(schema).map_err(IndexerSchemaError::ParseError)?;
            let (other_type_names, _) = build_schema_types_set(&ast);
            type_names.extend(other_type_names);

            for def in ast.definitions.iter() {
                if let TypeSystemDefinition::Type(t) = def {
                    match &t.node.kind {
                        TypeKind::Object(o) => {
                            let obj_name = t.node.name.to_string();
                            let mut field_mapping = BTreeMap::new();
                            parsed_type_names.insert(t.node.name.to_string());
                            for field in &o.fields {
                                let directives::Virtual(is_no_table) =
                                    get_notable_directive_info(&field.node).unwrap();

                                if is_no_table {
                                    virtual_type_names.insert(obj_name.clone());
                                }

                                let field_name = field.node.name.to_string();

                                let field_typ_name =
                                    normalize_field_type_name(&field.node.ty.to_string());

                                let field_id = format!("{obj_name}.{field_name}");

                                parsed_type_names.insert(field_name.clone());
                                field_mapping.insert(field_name, field_typ_name.clone());
                                field_type_optionality.insert(
                                    field_id.clone(),
                                    field.node.ty.node.nullable,
                                );
                                field_type_mappings.insert(field_id, field_typ_name);
                            }
                            object_field_mappings.insert(obj_name, field_mapping);
                        }
                        TypeKind::Enum(e) => {
                            let name = t.node.name.to_string();
                            virtual_type_names.insert(name.clone());
                            enum_names.insert(name.clone());

                            for val in &e.values {
                                let val_name = &val.node.value.to_string();
                                let val_id = format!("{name}.{val_name}");
                                field_type_mappings.insert(val_id, name.to_string());
                            }
                        }
                        TypeKind::Union(u) => {
                            let union_name = t.node.name.to_string();
                            union_names.insert(union_name.clone());

                            let member_count = u.members.len();
                            let virtual_member_count = u
                                .members
                                .iter()
                                .map(|m| {
                                    let member_name = m.node.to_string();
                                    if virtual_type_names.contains(&member_name) {
                                        1
                                    } else {
                                        0
                                    }
                                })
                                .sum::<usize>();

                            let mut has_virtual_member = false;

                            u.members.iter().for_each(|m| {
                                let member_name = m.node.to_string();
                                if let Some(name) = virtual_type_names.get(&member_name) {
                                    virtual_type_names.insert(name.to_owned());
                                    has_virtual_member = true;
                                }
                            });

                            if has_virtual_member {
                                virtual_type_names.insert(union_name.clone());

                                // All members of a union must all be regualar or virtual
                                if virtual_member_count != member_count {
                                    let e = format!("Union({union_name})'s members are not all virtual");
                                    return Err(
                                        IndexerSchemaError::InconsistentVirtualUnion(e),
                                    );
                                }
                            }
                        }
                        _ => {
                            return Err(IndexerSchemaError::UnsupportedTypeKind);
                        }
                    }
                }
            }
        }

        Ok(Self {
            namespace: namespace.to_string(),
            identifier: identifier.to_string(),
            is_native,
            type_names,
            union_names,
            object_field_mappings,
            enum_names,
            virtual_type_names,
            parsed_type_names,
            field_type_mappings,
            scalar_names,
            ast,
            field_type_optionality,
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
        self.virtual_type_names.contains(name) && !self.is_enum_type(name)
    }

    /// Whether the given field type name is an enum type.
    pub fn is_enum_type(&self, name: &str) -> bool {
        self.enum_names.contains(name)
    }

    /// Whether the given field type name is a union type.
    pub fn is_union_type(&self, name: &str) -> bool {
        self.union_names.contains(name)
    }

    /// Whether the parse schema contains the given type name.
    pub fn has_type(&self, name: &str) -> bool {
        self.type_names.contains(name)
    }
}

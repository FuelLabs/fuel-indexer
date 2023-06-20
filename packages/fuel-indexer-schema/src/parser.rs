use crate::{utils::*, IndexerSchemaError, IndexerSchemaResult};
use async_graphql_parser::{
    parse_schema,
    types::{
        EnumType, FieldDefinition, ObjectType, ServiceDocument, TypeKind,
        TypeSystemDefinition, UnionType,
    },
};
use fuel_indexer_database_types::directives;
use fuel_indexer_lib::ExecutionSource;
use fuel_indexer_types::graphql::GraphQLSchema;
use std::collections::{BTreeMap, HashMap, HashSet};

/// A wrapper object used to encapsulate a lot of the boilerplate logic related
/// to parsing schema, creating mappings of types, fields, objects, etc.
#[derive(Debug, Clone)]
pub struct ParsedGraphQLSchema {
    /// Namespace of the indexer.
    namespace: String,

    /// Identifier of the indexer.
    identifier: String,

    /// Indexer method of execution.
    exec_source: ExecutionSource,

    /// All unique names of types in the schema (whether objects, enums, or scalars).
    type_names: HashSet<String>,

    /// Mapping of object names to objects.
    objects: HashMap<String, ObjectType>,

    /// All unique names of enums in the schema.
    enum_names: HashSet<String>,

    /// Mapping of enum names to enum objects.
    enums: HashMap<String, EnumType>,

    /// All unique names of union types in the schema.
    union_names: HashSet<String>,

    /// Mapping of union names to union objects.
    unions: HashMap<String, UnionType>,

    // FIXME: Can't be private due to the `register_queryroot_fields` hack?
    /// All objects and their field names and types, indexed by object name.
    pub object_field_mappings: HashMap<String, BTreeMap<String, String>>,

    /// All unique names of types for which tables should _not_ be created.
    virtual_type_names: HashSet<String>,

    /// All unique names of types that have already been parsed.
    parsed_type_names: HashSet<String>,

    /// A mapping of fully qualified field names to their field types.
    field_type_mappings: HashMap<String, String>,

    /// All unique names of scalar types in the schema.
    scalar_names: HashSet<String>,

    // A mapping of fully qualified field names to their respective optionalities.
    field_type_optionality: HashMap<String, bool>,

    /// The parsed schema.
    ast: ServiceDocument,

    /// Mapping of fully qualified field names to their `FieldDefinition`
    field_defs: HashMap<String, FieldDefinition>,

    /// GraphQL schema content.
    schema: GraphQLSchema,

    // FIXME: How tf to calculate this using `get_column_type` and such?
    /// All unique names of foreign key types in the schema.
    foreign_key_mappings: HashMap<String, HashMap<String, (String, String)>>,
}

impl Default for ParsedGraphQLSchema {
    fn default() -> Self {
        let ast = parse_schema(BASE_SCHEMA)
            .map_err(IndexerSchemaError::ParseError)
            .expect("Bad schema");

        Self {
            namespace: "".to_string(),
            identifier: "".to_string(),
            exec_source: ExecutionSource::Wasm,
            type_names: HashSet::new(),
            enum_names: HashSet::new(),
            union_names: HashSet::new(),
            objects: HashMap::new(),
            enums: HashMap::new(),
            unions: HashMap::new(),
            virtual_type_names: HashSet::new(),
            parsed_type_names: HashSet::new(),
            field_type_mappings: HashMap::new(),
            object_field_mappings: HashMap::new(),
            scalar_names: HashSet::new(),
            field_defs: HashMap::new(),
            field_type_optionality: HashMap::new(),
            foreign_key_mappings: HashMap::new(),
            ast,
            schema: GraphQLSchema::default(),
        }
    }
}

impl ParsedGraphQLSchema {
    /// Create a new ParsedGraphQLSchema.
    pub fn new(
        namespace: &str,
        identifier: &str,
        exec_source: ExecutionSource,
        schema: Option<&GraphQLSchema>,
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
        let mut objects = HashMap::new();
        let mut enums = HashMap::new();
        let mut unions = HashMap::new();
        let mut field_defs = HashMap::new();
        let mut field_type_optionality = HashMap::new();
        let mut foreign_key_mappings = HashMap::new();

        // Parse _everything_ in the GraphQL schema
        if let Some(schema) = schema {
            ast =
                parse_schema(schema.schema()).map_err(IndexerSchemaError::ParseError)?;
            let (other_type_names, _) = build_schema_types_set(&ast);
            type_names.extend(other_type_names);

            for def in ast.definitions.iter() {
                if let TypeSystemDefinition::Type(t) = def {
                    match &t.node.kind {
                        TypeKind::Object(o) => {
                            let obj_name = t.node.name.to_string();
                            objects.insert(obj_name.clone(), o.clone());
                            let mut field_mapping = BTreeMap::new();
                            parsed_type_names.insert(t.node.name.to_string());
                            for field in &o.fields {
                                let directives::Virtual(is_virtual) =
                                    get_notable_directive_info(&field.node).unwrap();

                                if is_virtual {
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
                                field_type_mappings
                                    .insert(field_id.clone(), field_typ_name);
                                field_defs.insert(field_id, field.node.clone());
                            }
                            object_field_mappings.insert(obj_name, field_mapping);
                        }
                        TypeKind::Enum(e) => {
                            let name = t.node.name.to_string();
                            enums.insert(name.clone(), e.clone());
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
                            unions.insert(union_name.clone(), u.clone());
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

                            // These member fields are already cached under their respective object names, but
                            // we also need to cache them under this derived union name.
                            u.members.iter().for_each(|m| {
                                let member_name = m.node.to_string();
                                let member_obj = objects.get(&member_name).unwrap();
                                member_obj.fields.iter().for_each(|f| {
                                    let field_id =
                                        format!("{}.{}", union_name, f.node.name);
                                    field_defs.insert(field_id, f.node.clone());
                                });
                            });

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
            exec_source,
            type_names,
            union_names,
            objects,
            enums,
            unions,
            field_defs,
            foreign_key_mappings,
            object_field_mappings,
            enum_names,
            virtual_type_names,
            parsed_type_names,
            field_type_mappings,
            scalar_names,
            field_type_optionality,
            schema: schema.map(|s| s.clone()).unwrap_or_default(),
            ast,
        })
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub fn exec_source(&self) -> &ExecutionSource {
        &self.exec_source
    }

    pub fn type_names(&self) -> &HashSet<String> {
        &self.type_names
    }

    pub fn enum_names(&self) -> &HashSet<String> {
        &self.enum_names
    }

    pub fn union_names(&self) -> &HashSet<String> {
        &self.union_names
    }

    pub fn objects(&self) -> &HashMap<String, ObjectType> {
        &self.objects
    }

    pub fn enums(&self) -> &HashMap<String, EnumType> {
        &self.enums
    }

    pub fn unions(&self) -> &HashMap<String, UnionType> {
        &self.unions
    }

    pub fn virtual_type_names(&self) -> &HashSet<String> {
        &self.virtual_type_names
    }

    pub fn parsed_type_names(&self) -> &HashSet<String> {
        &self.parsed_type_names
    }

    pub fn field_type_mappings(&self) -> &HashMap<String, String> {
        &self.field_type_mappings
    }

    pub fn scalar_names(&self) -> &HashSet<String> {
        &self.scalar_names
    }

    pub fn field_type_optionality(&self) -> &HashMap<String, bool> {
        &self.field_type_optionality
    }

    pub fn ast(&self) -> &ServiceDocument {
        &self.ast
    }

    pub fn schema(&self) -> &GraphQLSchema {
        &self.schema
    }

    pub fn foreign_key_mappings(
        &self,
    ) -> &HashMap<String, HashMap<String, (String, String)>> {
        &self.foreign_key_mappings
    }

    pub fn object_field_mappings(&self) -> &HashMap<String, BTreeMap<String, String>> {
        &self.object_field_mappings
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

    /// All objects from which SQL tables can be created.
    pub fn indexable_objects(&self) -> Vec<ObjectType> {
        self.objects
            .iter()
            .filter(|(name, _)| !self.virtual_type_names.contains(*name))
            .map(|(_, obj)| obj.clone())
            .collect()
    }

    /// All fields from which SQL columns can be created.
    pub fn fields_for_columns(&self) -> Vec<FieldDefinition> {
        self.field_defs
            .iter()
            .filter(|(_, field)| !self.is_enum_type(&field.ty.to_string()))
            .map(|(_, field)| field.clone())
            .collect()
    }

    /// Return the GraphQL type for a given field name.
    pub fn field_type(&self, cond: &str, name: &str) -> Option<&String> {
        match self.object_field_mappings.get(cond) {
            Some(fieldset) => fieldset.get(name),
            _ => {
                let tablename = normalize_field_type_name(cond);
                match self.object_field_mappings.get(&tablename) {
                    Some(fieldset) => fieldset.get(name),
                    _ => None,
                }
            }
        }
    }

    /// Ensure the given type is included in this `Schema`'s types
    pub fn has_type(&self, name: &str) -> bool {
        self.type_names.contains(name)
    }
}

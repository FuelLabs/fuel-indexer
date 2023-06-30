use crate::{
    fully_qualified_namespace,
    graphql::{
        extract_foreign_key_info, field_id, GraphQLSchema, GraphQLSchemaValidator,
        BASE_SCHEMA,
    },
    ExecutionSource,
};
use async_graphql_parser::{
    parse_schema,
    types::{
        FieldDefinition, ObjectType, ServiceDocument, TypeDefinition, TypeKind,
        TypeSystemDefinition,
    },
};

use std::collections::{BTreeMap, HashMap, HashSet};
use thiserror::Error;

/// Result type returned by parsing GraphQL schema.
pub type ParsedResult<T> = Result<T, ParsedError>;

/// Error type returned by parsing GraphQL schema.
#[derive(Error, Debug)]
pub enum ParsedError {
    #[error("Generic error")]
    Generic,
    #[error("GraphQL parser error: {0:?}")]
    ParseError(#[from] async_graphql_parser::Error),
    #[error("This TypeKind is unsupported.")]
    UnsupportedTypeKind,
    #[error("List types are unsupported.")]
    ListTypesUnsupported,
    #[error("Inconsistent use of virtual union types. {0:?}")]
    InconsistentVirtualUnion(String),
}

/// Represents metadata related to a many-to-many relationship in the GraphQL schema.
#[derive(Debug, Clone)]
pub struct JoinTableItem {
    /// Name of the join table
    pub table_name: String,

    /// `TypeDefinition` name on which join relationship was found.
    pub local_table_name: String,

    /// Name of local column on which to join.
    ///
    /// This is always `id` for now.
    pub column_name: String,

    /// `TypeDefinition` name to which join references.
    pub ref_table_name: String,

    /// Name of the column on the referenced table to which to join.
    ///
    /// This is always `id` for now.
    pub ref_column_name: String,

    /// Type of the column on the referenced table to which to join.
    ///
    /// This is always `ColumnType::UInt8` for now.
    pub ref_column_type: String,
}

impl JoinTableItem {
    pub fn new(local_table_name: &str, ref_table_name: &str) -> Self {
        let local_table_name = local_table_name.to_string().to_lowercase();
        let ref_table_name = ref_table_name.to_string().to_lowercase();

        Self {
            table_name: format!("{local_table_name}s_{ref_table_name}s"),
            local_table_name,
            column_name: "id".to_string(),
            ref_table_name,
            ref_column_name: "id".to_string(),
            ref_column_type: "ID".to_string(),
        }
    }
}

/// Given a GraphQL document, return a two `HashSet`s - one for each
/// unique field type, and one for each unique directive.
pub fn build_schema_types_set(
    ast: &ServiceDocument,
) -> (HashSet<String>, HashSet<String>) {
    let types: HashSet<String> = ast
        .definitions
        .iter()
        .filter_map(|def| {
            if let TypeSystemDefinition::Type(typ) = def {
                Some(&typ.node)
            } else {
                None
            }
        })
        .map(|t| t.name.to_string())
        .collect();

    let directives = ast
        .definitions
        .iter()
        .filter_map(|def| {
            if let TypeSystemDefinition::Directive(dir) = def {
                Some(dir.node.name.to_string())
            } else {
                None
            }
        })
        .collect();

    (types, directives)
}

/// A wrapper object used to encapsulate a lot of the boilerplate logic related
/// to parsing schema, creating mappings of types, fields, objects, etc.
//
// Ideally `ParsedGraphQLSchema` prevents from having to manually parse `async_graphql_parser`
// `TypeDefinition`s in order to get metadata on the types (e.g., Is a foreign key? is a virtual type?
// and so on).
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

    /// Mapping of union names to unions.
    unions: HashMap<String, TypeDefinition>,

    /// All unique names of enums in the schema.
    enum_names: HashSet<String>,

    /// All unique names of union types in the schema.
    union_names: HashSet<String>,

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

    /// A mapping of fully qualified field names to their respective optionalities.
    field_type_optionality: HashMap<String, bool>,

    /// The parsed schema.
    ast: ServiceDocument,

    /// Mapping of fully qualified field names to their `FieldDefinition` and `TypeDefinition` name.
    //
    // We keep the `TypeDefinition` name so that we can know what type of object the field belongs to.
    field_defs: HashMap<String, (FieldDefinition, String)>,

    /// GraphQL schema content.
    schema: GraphQLSchema,

    /// All unique names of foreign key types in the schema.
    foreign_key_mappings: HashMap<String, HashMap<String, (String, String)>>,

    /// All type definitions in the schema.
    type_defs: HashMap<String, TypeDefinition>,

    /// `FieldDefinition` names in the GraphQL that are a `List` type.
    list_field_types: HashSet<String>,

    /// `TypeDefinition`s that contain a `FieldDefinition` which is a `List` type.
    list_type_defs: HashMap<String, TypeDefinition>,

    /// Metadata related to many-to-many relationships in the GraphQL schema.
    join_table_info: HashMap<String, JoinTableItem>,
}

impl Default for ParsedGraphQLSchema {
    fn default() -> Self {
        let ast = parse_schema(BASE_SCHEMA)
            .map_err(ParsedError::ParseError)
            .expect("Bad schema");

        Self {
            namespace: "".to_string(),
            identifier: "".to_string(),
            exec_source: ExecutionSource::Wasm,
            type_names: HashSet::new(),
            enum_names: HashSet::new(),
            union_names: HashSet::new(),
            objects: HashMap::new(),
            virtual_type_names: HashSet::new(),
            parsed_type_names: HashSet::new(),
            field_type_mappings: HashMap::new(),
            object_field_mappings: HashMap::new(),
            scalar_names: HashSet::new(),
            field_defs: HashMap::new(),
            field_type_optionality: HashMap::new(),
            foreign_key_mappings: HashMap::new(),
            type_defs: HashMap::new(),
            ast,
            schema: GraphQLSchema::default(),
            list_field_types: HashSet::new(),
            list_type_defs: HashMap::new(),
            unions: HashMap::new(),
            join_table_info: HashMap::new(),
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
    ) -> ParsedResult<Self> {
        let mut ast = parse_schema(BASE_SCHEMA).map_err(ParsedError::ParseError)?;
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
        let mut field_defs = HashMap::new();
        let mut field_type_optionality = HashMap::new();
        let mut foreign_key_mappings: HashMap<String, HashMap<String, (String, String)>> =
            HashMap::new();
        let mut type_defs = HashMap::new();
        let mut list_field_types = HashSet::new();
        let mut list_type_defs = HashMap::new();
        let mut unions = HashMap::new();
        let mut join_table_info = HashMap::new();

        // Parse _everything_ in the GraphQL schema
        if let Some(schema) = schema {
            ast = parse_schema(schema.schema()).map_err(ParsedError::ParseError)?;
            let (other_type_names, _) = build_schema_types_set(&ast);
            type_names.extend(other_type_names);

            for def in ast.definitions.iter() {
                if let TypeSystemDefinition::Type(t) = def {
                    match &t.node.kind {
                        TypeKind::Object(o) => {
                            let obj_name = t.node.name.to_string();

                            type_defs.insert(obj_name.clone(), t.node.clone());
                            objects.insert(obj_name.clone(), o.clone());
                            parsed_type_names.insert(t.node.name.to_string());

                            let mut field_mapping = BTreeMap::new();
                            for field in &o.fields {
                                let field_name = field.node.name.to_string();
                                let fid = field_id(&obj_name, &field_name);

                                if field.node.ty.to_string().contains('[')
                                    && field.node.ty.to_string().contains(']')
                                {
                                    let fid = field_id(&obj_name, &field_name);
                                    list_field_types.insert(fid);

                                    list_type_defs
                                        .insert(obj_name.clone(), t.node.clone());
                                }

                                let is_virtual = field
                                    .node
                                    .directives
                                    .iter()
                                    .any(|d| d.node.name.to_string() == "virtual");

                                if is_virtual {
                                    virtual_type_names.insert(obj_name.clone());
                                }

                                // Manual version of `ParsedGraphQLSchema::is_possible_foreign_key`
                                if parsed_type_names.contains(
                                    &field
                                        .node
                                        .ty
                                        .to_string()
                                        .replace(['[', ']', '!'], ""),
                                ) && !scalar_names.contains(&field_name)
                                    && !enum_names.contains(&field_name)
                                    && !virtual_type_names.contains(&field_name)
                                {
                                    let (_ref_coltype, ref_colname, ref_tablename) =
                                        extract_foreign_key_info(
                                            &field.node,
                                            &field_type_mappings,
                                        );
                                        
                                        join_table_info.insert(
                                            obj_name.clone(),
                                            JoinTableItem::new(&obj_name, &ref_tablename),
                                        );

                                    let fk = foreign_key_mappings
                                        .get_mut(&t.node.name.to_string().to_lowercase());
                                    match fk {
                                        Some(fks_for_field) => {
                                            fks_for_field.insert(
                                                field.node.name.to_string(),
                                                (
                                                    field
                                                        .node
                                                        .ty
                                                        .to_string()
                                                        .replace(['[', ']', '!'], "")
                                                        .to_lowercase(),
                                                    ref_colname.clone(),
                                                ),
                                            );
                                        }
                                        None => {
                                            let fks_for_field = HashMap::from([(
                                                field.node.name.to_string(),
                                                (
                                                    field
                                                        .node
                                                        .ty
                                                        .to_string()
                                                        .replace(['[', ']', '!'], "")
                                                        .to_lowercase(),
                                                    ref_colname.clone(),
                                                ),
                                            )]);
                                            foreign_key_mappings.insert(
                                                t.node.name.to_string().to_lowercase(),
                                                fks_for_field,
                                            );
                                        }
                                    }
                                }

                                let field_typ_name = field
                                    .node
                                    .ty
                                    .to_string()
                                    .replace(['[', ']', '!'], "");

                                parsed_type_names.insert(field_name.clone());
                                field_mapping.insert(field_name, field_typ_name.clone());
                                field_type_optionality
                                    .insert(fid.clone(), field.node.ty.node.nullable);
                                field_type_mappings.insert(fid.clone(), field_typ_name);
                                field_defs
                                    .insert(fid, (field.node.clone(), obj_name.clone()));
                            }
                            object_field_mappings.insert(obj_name, field_mapping);
                        }
                        TypeKind::Enum(e) => {
                            let name = t.node.name.to_string();
                            type_defs.insert(name.clone(), t.node.clone());

                            virtual_type_names.insert(name.clone());
                            enum_names.insert(name.clone());

                            for val in &e.values {
                                let val_name = &val.node.value.to_string();
                                let val_id = format!("{}.{val_name}", name.clone());
                                object_field_mappings
                                    .entry(name.clone())
                                    .or_insert_with(BTreeMap::new)
                                    .insert(val_name.to_string(), name.clone());
                                field_type_mappings.insert(val_id, name.to_string());
                            }
                        }
                        TypeKind::Union(u) => {
                            let union_name = t.node.name.to_string();

                            parsed_type_names.insert(union_name.clone());
                            type_defs.insert(union_name.clone(), t.node.clone());
                            unions.insert(union_name.clone(), t.node.clone());

                            union_names.insert(union_name.clone());

                            GraphQLSchemaValidator::check_derived_union_is_well_formed(
                                &t.node,
                                &mut virtual_type_names,
                            );

                            u.members.iter().for_each(|m| {
                                let member_name = m.node.to_string();
                                if let Some(name) = virtual_type_names.get(&member_name) {
                                    virtual_type_names.insert(name.to_owned());
                                }
                            });

                            // These member fields are already cached under their respective object names, but
                            // we also need to cache them under this derived union name.
                            u.members.iter().for_each(|m| {
                                let member_name = m.node.to_string();
                                let member_obj = objects.get(&member_name).unwrap();
                                member_obj.fields.iter().for_each(|f| {
                                    let fid =
                                        field_id(&union_name, &f.node.name.to_string());
                                    field_defs.insert(
                                        fid.clone(),
                                        (f.node.clone(), member_name.clone()),
                                    );

                                    field_type_mappings.insert(
                                        fid.clone(),
                                        f.node
                                            .ty
                                            .to_string()
                                            .replace(['[', ']', '!'], ""),
                                    );

                                    object_field_mappings
                                        .entry(union_name.clone())
                                        .or_insert_with(BTreeMap::new)
                                        .insert(
                                            f.node.name.to_string(),
                                            f.node
                                                .ty
                                                .to_string()
                                                .replace(['[', ']', '!'], ""),
                                        );

                                    field_type_optionality
                                        .insert(fid, f.node.ty.node.nullable);
                                });
                            });
                        }
                        _ => {
                            return Err(ParsedError::UnsupportedTypeKind);
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
            field_defs,
            foreign_key_mappings,
            object_field_mappings,
            enum_names,
            virtual_type_names,
            parsed_type_names,
            field_type_mappings,
            scalar_names,
            field_type_optionality,
            schema: schema.cloned().unwrap(),
            ast,
            type_defs,
            list_field_types,
            list_type_defs,
            unions,
            join_table_info,
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

    pub fn objects(&self) -> &HashMap<String, ObjectType> {
        &self.objects
    }

    pub fn field_type_mappings(&self) -> &HashMap<String, String> {
        &self.field_type_mappings
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

    pub fn type_defs(&self) -> &HashMap<String, TypeDefinition> {
        &self.type_defs
    }

    pub fn field_defs(&self) -> &HashMap<String, (FieldDefinition, String)> {
        &self.field_defs
    }

    pub fn foreign_key_mappings(
        &self,
    ) -> &HashMap<String, HashMap<String, (String, String)>> {
        &self.foreign_key_mappings
    }

    pub fn object_field_mappings(&self) -> &HashMap<String, BTreeMap<String, String>> {
        &self.object_field_mappings
    }

    pub fn join_table_info(&self) -> &HashMap<String, JoinTableItem> {
        &self.join_table_info
    }

    /// Return the `TypeDefinition` associated with a given union name.
    pub fn get_union(&self, name: &str) -> Option<&TypeDefinition> {
        self.unions.get(name)
    }

    /// Return a list of all non-enum type definitions.
    pub fn non_enum_typdefs(&self) -> Vec<(&String, &TypeDefinition)> {
        self.type_defs
            .iter()
            .filter(|(_, t)| !matches!(&t.kind, TypeKind::Enum(_)))
            .collect()
    }

    /// Whether the given field type name is a possible foreign key.
    pub fn is_possible_foreign_key(&self, name: &str) -> bool {
        self.parsed_type_names.contains(name)
            && !self.scalar_names.contains(name)
            && !self.is_enum_typedef(name)
            && !self.is_virtual_typedef(name)
    }

    /// Whether the given field type name is a type from which tables are not created.
    pub fn is_virtual_typedef(&self, name: &str) -> bool {
        self.virtual_type_names.contains(name) && !self.is_enum_typedef(name)
    }

    /// Whether the given field type name is an enum type.
    pub fn is_enum_typedef(&self, name: &str) -> bool {
        self.enum_names.contains(name)
    }

    /// Whether the given field type name is a list type.
    pub fn is_list_field_type(&self, name: &str) -> bool {
        self.list_field_types.contains(name)
    }

    /// Whether a given `TypeDefinition` contains a field that is a list type.
    pub fn is_list_typedef(&self, name: &str) -> bool {
        self.list_type_defs.contains_key(name)
    }

    /// Whether the given field type name is a union type.
    pub fn is_union_typedef(&self, name: &str) -> bool {
        self.union_names.contains(name)
    }

    /// Return the GraphQL type for a given field name.
    pub fn field_type(&self, cond: &str, name: &str) -> Option<&String> {
        match self.object_field_mappings.get(cond) {
            Some(fieldset) => fieldset.get(name),
            _ => {
                let tablename = cond.replace(['[', ']', '!'], "");
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

    /// Fully qualified GraphQL namespace for indexer.
    pub fn fully_qualified_namespace(&self) -> String {
        fully_qualified_namespace(&self.namespace, &self.identifier)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_caches_all_related_typedefs_when_instantiated() {
        let schema = r#"
enum AccountLabel {
    PRIMARY
    SECONDARY
}

type Account {
    id: ID!
    address: Address!
    label: AccountLabel
}

type User {
    id: ID!
    account: Account!
    username: Charfield!
}

type Loser {
    id: ID!
    account: Account!
    age: UInt8!
}

type Metadata {
    count: UInt8! @virtual
}

union Person = User | Loser
"#;

        let parsed = ParsedGraphQLSchema::new(
            "test",
            "test",
            ExecutionSource::Wasm,
            Some(&GraphQLSchema::new(schema.to_string())),
        );

        assert!(parsed.is_ok());

        let parsed = parsed.unwrap();

        assert!(parsed.has_type("Account"));
        assert!(parsed.has_type("User"));
        assert!(parsed.is_possible_foreign_key("Account"));
        assert!(parsed.is_virtual_typedef("Metadata"));
        assert!(parsed.is_enum_typedef("AccountLabel"));
        assert!(parsed
            .field_type_optionality()
            .contains_key("Account.label"));

        assert!(parsed.is_union_typedef("Person"));
    }
}

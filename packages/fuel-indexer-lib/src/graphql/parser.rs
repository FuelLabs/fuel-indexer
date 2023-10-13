//! # fuel_indexer_lib::parser
//!
//! A utility used to help parse and cache various components of indexer
//! GraphQL schema. This is meant to be a productivity tool for project devs.

use crate::{
    fully_qualified_namespace,
    graphql::{
        extract_foreign_key_info, field_id, field_type_name,
        inject_internal_types_into_document, is_list_type, list_field_type_name,
        GraphQLSchema, GraphQLSchemaValidator, IdCol, BASE_SCHEMA,
    },
    join_table_name, ExecutionSource,
};
use async_graphql_parser::{
    parse_schema,
    types::{
        EnumType, FieldDefinition, ObjectType, ServiceDocument, TypeDefinition, TypeKind,
        TypeSystemDefinition, UnionType,
    },
};
use async_graphql_value::ConstValue;

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
    #[error("Union member not found in parsed TypeDefintions. {0:?}")]
    UnionMemberNotFound(String),
}

/// Represents metadata related to a many-to-many relationship in the GraphQL schema.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct JoinTableMeta {
    /// The `TypeDefinition` on which the `FieldDefinition` with a list type is defined.
    parent: JoinTableRelation,

    /// The `TypeDefinition` who's inner content type is a list of foreign keys.
    child: JoinTableRelation,
}

impl JoinTableMeta {
    pub fn parent(&self) -> &JoinTableRelation {
        &self.parent
    }

    pub fn child(&self) -> &JoinTableRelation {
        &self.child
    }
}

/// Represents a relationship between two `TypeDefinition`s in the GraphQL schema.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct JoinTableRelation {
    /// Whether this is the parent or the child in the join.
    pub relation_type: JoinTableRelationType,

    /// Name of the `TypeDefinition` associated with this join.
    pub typedef_name: String,

    /// Name of the column in the join table.
    pub column_name: String,

    /// Position of the child in the join table.
    pub child_position: Option<usize>,
}

/// Type of join table relationship.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum JoinTableRelationType {
    /// `TypeDefinition` on which the list type is defined.
    Parent,

    /// A `Child` in this case, is a `FieldDefinition` on a `TypeDefinition` that
    /// contains a list type, whose inner content type is a foreign key reference.
    Child,
}

impl JoinTableMeta {
    /// Create a new `JoinTableMeta`.
    pub fn new(
        parent_typedef_name: &str,
        parent_column_name: &str,
        child_typedef_name: &str,
        child_column_name: &str,
        child_position: Option<usize>,
    ) -> Self {
        Self {
            parent: JoinTableRelation {
                relation_type: JoinTableRelationType::Parent,
                typedef_name: parent_typedef_name.to_string(),
                column_name: parent_column_name.to_string(),
                child_position,
            },
            child: JoinTableRelation {
                relation_type: JoinTableRelationType::Child,
                typedef_name: child_typedef_name.to_string(),
                column_name: child_column_name.to_string(),
                child_position: None,
            },
        }
    }

    pub fn table_name(&self) -> String {
        join_table_name(&self.parent_table_name(), &self.child_table_name())
    }

    pub fn parent_table_name(&self) -> String {
        self.parent.typedef_name.to_lowercase()
    }

    pub fn parent_column_name(&self) -> String {
        self.parent.column_name.clone()
    }

    pub fn child_table_name(&self) -> String {
        self.child.typedef_name.to_lowercase()
    }

    pub fn child_column_name(&self) -> String {
        self.child.column_name.clone()
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

/// A wrapper object used to keep track of the order of a `FieldDefinition` in an object ` TypeDefinition`.
#[derive(Debug, Clone)]
pub struct OrderedField(pub FieldDefinition, pub usize);

/// A wrapper object used to encapsulate a lot of the boilerplate logic related
/// to parsing schema, creating mappings of types, fields, objects, etc.
///
/// Ideally `ParsedGraphQLSchema` prevents from having to manually parse `async_graphql_parser`
/// `TypeDefinition`s in order to get metadata on the types (e.g., Is a foreign key? is a virtual type?
/// and so on).
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

    /// Mapping of lowercase `TypeDefinition` names to their actual `TypeDefinition` names.
    ///
    /// Used to refer to top-level entities in GraphQL queries.
    typedef_names_to_types: HashMap<String, String>,

    /// Mapping of object names to objects.
    objects: HashMap<String, ObjectType>,

    /// Mapping of union names to unions.
    unions: HashMap<String, TypeDefinition>,

    /// All unique names of enums in the schema.
    enum_names: HashSet<String>,

    /// All unique names of union types in the schema.
    union_names: HashSet<String>,

    /// All objects and their field names and types, indexed by object name.
    object_field_mappings: HashMap<String, BTreeMap<String, String>>,

    /// All unique names of types for which tables should _not_ be created.
    virtual_type_names: HashSet<String>,

    /// All unique names of types that have already been parsed.
    parsed_typedef_names: HashSet<String>,

    /// Mapping of fully qualified field names to their field types.
    field_type_mappings: HashMap<String, String>,

    /// All unique names of scalar types in the schema.
    scalar_names: HashSet<String>,

    /// A mapping of fully qualified field names to their respective optionalities.
    field_type_optionality: HashMap<String, bool>,

    /// Mapping of fully qualified field names to their `FieldDefinition` and `TypeDefinition` name.
    ///
    /// We keep the `TypeDefinition` name so that we can know what type of object the field belongs to.
    field_defs: HashMap<String, (FieldDefinition, String)>,

    /// All unique names of foreign key types in the schema.
    foreign_key_mappings: HashMap<String, HashMap<String, (String, String)>>,

    /// All type definitions in the schema.
    type_defs: HashMap<String, TypeDefinition>,

    /// `FieldDefinition` names in the GraphQL that are a `List` type.
    list_field_types: HashSet<String>,

    /// `TypeDefinition`s that contain a `FieldDefinition` which is a `List` type.
    list_type_defs: HashMap<String, TypeDefinition>,

    /// Metadata related to many-to-many relationships in the GraphQL schema.
    ///
    /// Many-to-many (m2m) relationships are created when a `FieldDefinition` contains a
    /// list type, whose inner content type is a foreign key reference to another `TypeDefinition`.
    join_table_meta: HashMap<String, Vec<JoinTableMeta>>,

    /// A mapping of object `TypeDefinition` names, and their respective `FieldDefinition`s - including
    /// the order of that `FieldDefinition` in the object.
    ///
    /// When creating these derived object `TypeDefinition`s from the members of a union `TypeDefinition`, we
    /// need to preserve the order of the fields as they appear in their original object `TypeDefinitions`.
    /// This allows us to create SQL tables where the columns are ordered - mirroring the order of the fields
    /// on the object `TypeDefinition` derived from a union.
    object_ordered_fields: HashMap<String, Vec<OrderedField>>,

    /// The version of the schema.
    version: String,

    /// Internal types. These types should not be added to a
    /// database in any way; they are used to augment repsonses for introspection queries.
    internal_types: HashSet<String>,
}

impl Default for ParsedGraphQLSchema {
    fn default() -> Self {
        Self {
            namespace: "".to_string(),
            identifier: "".to_string(),
            exec_source: ExecutionSource::Wasm,
            type_names: HashSet::new(),
            typedef_names_to_types: HashMap::new(),
            enum_names: HashSet::new(),
            union_names: HashSet::new(),
            objects: HashMap::new(),
            virtual_type_names: HashSet::new(),
            parsed_typedef_names: HashSet::new(),
            field_type_mappings: HashMap::new(),
            object_field_mappings: HashMap::new(),
            scalar_names: HashSet::new(),
            field_defs: HashMap::new(),
            field_type_optionality: HashMap::new(),
            foreign_key_mappings: HashMap::new(),
            type_defs: HashMap::new(),
            list_field_types: HashSet::new(),
            list_type_defs: HashMap::new(),
            unions: HashMap::new(),
            join_table_meta: HashMap::new(),
            object_ordered_fields: HashMap::new(),
            version: String::default(),
            internal_types: HashSet::new(),
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
        let base_type_names = {
            let base_ast = parse_schema(BASE_SCHEMA)?;
            let mut base_decoder = SchemaDecoder::new();
            base_decoder.decode_service_document(base_ast)?;
            base_decoder.parsed_graphql_schema.type_names
        };

        let mut decoder = SchemaDecoder::new();

        if let Some(schema) = schema {
            // Parse _everything_ in the GraphQL schema
            let mut ast = parse_schema(schema.schema())?;

            ast = inject_internal_types_into_document(ast, &base_type_names);

            decoder.decode_service_document(ast)?;

            decoder.parsed_graphql_schema.namespace = namespace.to_string();
            decoder.parsed_graphql_schema.identifier = identifier.to_string();
            decoder.parsed_graphql_schema.exec_source = exec_source.clone();
            decoder.parsed_graphql_schema.version = schema.version.clone();
        };

        let mut result = decoder.get_parsed_schema();

        result.type_names.extend(base_type_names.clone());
        result.scalar_names.extend(base_type_names);

        Ok(result)
    }

    /// Namespace of the indexer.
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Identifier of the indexer.
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    /// Indexer method of execution.
    pub fn exec_source(&self) -> &ExecutionSource {
        &self.exec_source
    }

    /// Mapping of object names to objects.    
    pub fn objects(&self) -> &HashMap<String, ObjectType> {
        &self.objects
    }

    /// Mapping of fully qualified field names to their field types.
    pub fn field_type_mappings(&self) -> &HashMap<String, String> {
        &self.field_type_mappings
    }

    /// A mapping of fully qualified field names to their respective optionalities.
    pub fn field_type_optionality(&self) -> &HashMap<String, bool> {
        &self.field_type_optionality
    }

    /// All type definitions in the schema.
    pub fn type_defs(&self) -> &HashMap<String, TypeDefinition> {
        &self.type_defs
    }

    /// Mapping of fully qualified field names to their `FieldDefinition` and `TypeDefinition` name.
    pub fn field_defs(&self) -> &HashMap<String, (FieldDefinition, String)> {
        &self.field_defs
    }

    /// All unique names of foreign key types in the schema.
    pub fn foreign_key_mappings(
        &self,
    ) -> &HashMap<String, HashMap<String, (String, String)>> {
        &self.foreign_key_mappings
    }

    /// All objects and their field names and types, indexed by object name.
    pub fn object_field_mappings(&self) -> &HashMap<String, BTreeMap<String, String>> {
        &self.object_field_mappings
    }

    /// Metadata related to many-to-many relationships in the GraphQL schema.
    pub fn join_table_meta(&self) -> &HashMap<String, Vec<JoinTableMeta>> {
        &self.join_table_meta
    }

    pub fn object_ordered_fields(&self) -> &HashMap<String, Vec<OrderedField>> {
        &self.object_ordered_fields
    }

    /// Return the base scalar type for a given `FieldDefinition`.
    pub fn scalar_type_for(&self, f: &FieldDefinition) -> String {
        let typ_name = list_field_type_name(f);
        if self.is_list_field_type(&typ_name) {
            let typ_name = field_type_name(f);
            if self.is_possible_foreign_key(&typ_name) {
                let (ref_coltype, _ref_colname, _ref_tablename) =
                    extract_foreign_key_info(f, &self.field_type_mappings);

                return ref_coltype;
            } else if self.is_virtual_typedef(&typ_name) {
                return "Json".to_string();
            } else if self.is_enum_typedef(&typ_name) {
                return "String".to_string();
            } else {
                return typ_name;
            }
        }

        if self.is_possible_foreign_key(&typ_name) {
            let (ref_coltype, _ref_colname, _ref_tablename) =
                extract_foreign_key_info(f, &self.field_type_mappings);
            return ref_coltype;
        }

        if self.is_virtual_typedef(&typ_name) {
            return "Json".to_string();
        }

        if self.is_enum_typedef(&typ_name) {
            return "String".to_string();
        }

        typ_name
    }

    /// Return the `TypeDefinition` associated with a given union name.
    pub fn get_union(&self, name: &str) -> Option<&TypeDefinition> {
        self.unions.get(name)
    }

    /// Return a list of all type definitions that will have a record or table in
    /// the database; functionally, this means any non-enum or internal type defintions.
    pub fn storage_backed_typedefs(&self) -> Vec<(&String, &TypeDefinition)> {
        self.type_defs
            .iter()
            .filter(|(_, t)| {
                !matches!(&t.kind, TypeKind::Enum(_))
                    && !self.is_internal_typedef(t.name.node.as_str())
            })
            .collect()
    }

    /// Whether the given field type name is a possible foreign key.
    pub fn is_possible_foreign_key(&self, name: &str) -> bool {
        self.parsed_typedef_names.contains(name)
            && !self.scalar_names.contains(name)
            && !self.is_enum_typedef(name)
            && !self.is_virtual_typedef(name)
            && !self.is_internal_typedef(name)
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

    pub fn is_internal_typedef(&self, name: &str) -> bool {
        self.internal_types.contains(name)
    }

    /// Return the GraphQL type for a given `FieldDefinition` name.
    fn field_type(&self, cond: &str, name: &str) -> Option<&String> {
        match self.object_field_mappings().get(cond) {
            Some(fieldset) => fieldset.get(name),
            _ => {
                let tablename = cond.replace(['[', ']', '!'], "");
                match self.object_field_mappings().get(&tablename) {
                    Some(fieldset) => fieldset.get(name),
                    _ => None,
                }
            }
        }
    }

    /// Return the GraphQL type for a given `TypeDefinition` name.
    fn typedef_type(&self, name: &str) -> Option<&String> {
        self.typedef_names_to_types.get(name)
    }

    /// Return the GraphQL type for a given `FieldDefinition` or `TypeDefinition` name.
    ///
    /// This serves as a convenience function so that the caller doesn't have to
    /// worry about handling the case in which `cond` is not present; for example,
    /// `cond` is None when retrieving the type for a top-level entity in a query.
    pub fn graphql_type(&self, cond: Option<&String>, name: &str) -> Option<&String> {
        match cond {
            Some(c) => self.field_type(c, name),
            None => self.typedef_type(name),
        }
    }

    /// Ensure the given type is included in this `Schema`'s types
    pub fn has_type(&self, name: &str) -> bool {
        self.type_names.contains(name)
    }

    /// Fully qualified namespace for the indexer.
    pub fn fully_qualified_namespace(&self) -> String {
        fully_qualified_namespace(&self.namespace, &self.identifier)
    }

    /// Version of the schema.
    pub fn version(&self) -> &str {
        &self.version
    }
}

#[derive(Default)]
struct SchemaDecoder {
    parsed_graphql_schema: ParsedGraphQLSchema,
}

impl SchemaDecoder {
    fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    fn get_parsed_schema(self) -> ParsedGraphQLSchema {
        self.parsed_graphql_schema
    }

    /// Parse and decode the base GraphQL Schema
    fn decode_service_document(&mut self, ast: ServiceDocument) -> ParsedResult<()> {
        for def in ast.definitions.iter() {
            self.decode_type_system_definifion(def)?;
        }
        self.build_typedef_names_to_types();
        Ok(())
    }

    fn decode_type_system_definifion(
        &mut self,
        def: &TypeSystemDefinition,
    ) -> ParsedResult<()> {
        if let TypeSystemDefinition::Type(t) = def {
            let name = t.node.name.to_string();
            let node = t.node.clone();

            self.parsed_graphql_schema.type_names.insert(name.clone());

            self.parsed_graphql_schema
                .type_defs
                .insert(name.clone(), node.clone());

            match &t.node.kind {
                TypeKind::Object(o) => self.decode_object_type(name, node, o),
                TypeKind::Enum(e) => self.decode_enum_type(name, e),
                TypeKind::Union(u) => self.decode_union_type(name, node, u),
                TypeKind::Scalar => {
                    self.parsed_graphql_schema.scalar_names.insert(name.clone());
                }
                _ => {
                    return Err(ParsedError::UnsupportedTypeKind);
                }
            }
        }

        Ok(())
    }

    fn decode_enum_type(&mut self, name: String, e: &EnumType) {
        self.parsed_graphql_schema
            .virtual_type_names
            .insert(name.clone());
        self.parsed_graphql_schema.enum_names.insert(name.clone());

        for val in &e.values {
            let val_name = &val.node.value.to_string();
            let val_id = format!("{}.{val_name}", name.clone());
            self.parsed_graphql_schema
                .object_field_mappings
                .entry(name.clone())
                .or_default()
                .insert(val_name.to_string(), name.clone());
            self.parsed_graphql_schema
                .field_type_mappings
                .insert(val_id, name.to_string());
        }
    }

    fn decode_union_type(
        &mut self,
        union_name: String,
        node: TypeDefinition,
        u: &UnionType,
    ) {
        GraphQLSchemaValidator::check_disallowed_graphql_typedef_name(&union_name);

        self.parsed_graphql_schema
            .parsed_typedef_names
            .insert(union_name.clone());
        self.parsed_graphql_schema
            .unions
            .insert(union_name.clone(), node.clone());

        self.parsed_graphql_schema
            .union_names
            .insert(union_name.clone());

        GraphQLSchemaValidator::check_derived_union_virtuality_is_well_formed(
            &node,
            &mut self.parsed_graphql_schema.virtual_type_names,
        );

        // Ensure we're not creating duplicate join table metadata, else we'll
        // have issues trying to create duplicate `TypeIds` when constructing SQL tables.
        let mut processed_fields = HashSet::new();

        // Child position in the union is different than child position in the object.
        // In the object, you simply count the fields. However, in a union, you have to
        // count the distinct fields across all members of the union.
        let mut child_position = 0;

        let mut union_member_field_types = HashMap::new();

        u.members.iter().for_each(|m| {
            let member_name = m.node.to_string();
            if let Some(name) = self
                .parsed_graphql_schema
                .virtual_type_names
                .get(&member_name)
            {
                self.parsed_graphql_schema
                    .virtual_type_names
                    .insert(name.to_owned());
            }

            // Don't create many-to-many relationships for `TypeDefintions` that are themselves
            // members of union `TypeDefinition`s.
            if self
                .parsed_graphql_schema
                .join_table_meta
                .contains_key(&member_name)
            {
                self.parsed_graphql_schema
                    .join_table_meta
                    .remove(&member_name);
            }

            // Parse the many-to-many relationship metadata the same as we do for
            // `TypeKind::Object` above, just using each union member's fields.
            let member_obj = self
                .parsed_graphql_schema
                .objects
                .get(&member_name)
                .expect("Union member not found in parsed TypeDefinitions.");

            member_obj.fields.iter().for_each(|f| {
                let ftype = field_type_name(&f.node);
                let field_id = field_id(&union_name, &f.node.name.to_string());

                union_member_field_types
                    .entry(field_id.clone())
                    .or_insert(HashSet::new())
                    .insert(ftype.clone());

                GraphQLSchemaValidator::derived_field_type_is_consistent(
                    &union_name,
                    &f.node.name.to_string(),
                    union_member_field_types.get(&field_id).unwrap(),
                );

                if processed_fields.contains(&field_id) {
                    return;
                }

                processed_fields.insert(field_id.clone());

                // Manual foreign key check, same as above
                if self
                    .parsed_graphql_schema
                    .parsed_typedef_names
                    .contains(&ftype)
                    && !self.parsed_graphql_schema.scalar_names.contains(&ftype)
                    && !self.parsed_graphql_schema.enum_names.contains(&ftype)
                    && !self
                        .parsed_graphql_schema
                        .virtual_type_names
                        .contains(&ftype)
                    && !self.parsed_graphql_schema.internal_types.contains(&ftype)
                {
                    let (_ref_coltype, ref_colname, ref_tablename) =
                        extract_foreign_key_info(
                            &f.node,
                            &self.parsed_graphql_schema.field_type_mappings,
                        );

                    if is_list_type(&f.node) {
                        self.parsed_graphql_schema
                            .join_table_meta
                            .entry(union_name.clone())
                            .or_default()
                            .push(JoinTableMeta::new(
                                &union_name.to_lowercase(),
                                // The parent join column is _always_ `id: ID!`
                                IdCol::to_lowercase_str(),
                                &ref_tablename,
                                &ref_colname,
                                Some(child_position),
                            ));
                    }
                }

                child_position += 1;
            });
        });

        // These member fields are already cached under their respective object names, but
        // we also need to cache them under this derived union name.
        u.members.iter().for_each(|m| {
            let member_name = m.node.to_string();
            let member_obj =
                self.parsed_graphql_schema
                    .objects
                    .get(&member_name)
                    .unwrap_or_else(|| {
                        panic!(
                        "Union member not found in parsed TypeDefinitions: {member_name}")
                    });

            member_obj.fields.iter().for_each(|f| {
                let fid = field_id(&union_name, &f.node.name.to_string());
                self.parsed_graphql_schema
                    .field_defs
                    .insert(fid.clone(), (f.node.clone(), member_name.clone()));

                self.parsed_graphql_schema
                    .field_type_mappings
                    .insert(fid.clone(), field_type_name(&f.node));

                self.parsed_graphql_schema
                    .object_field_mappings
                    .entry(union_name.clone())
                    .or_default()
                    .insert(f.node.name.to_string(), field_type_name(&f.node));

                self.parsed_graphql_schema
                    .field_type_optionality
                    .insert(fid, f.node.ty.node.nullable);
            });
        });
    }

    fn decode_object_type(
        &mut self,
        obj_name: String,
        node: TypeDefinition,
        o: &ObjectType,
    ) {
        GraphQLSchemaValidator::check_disallowed_graphql_typedef_name(&obj_name);

        let is_internal = node
            .directives
            .iter()
            .any(|d| d.node.name.node == "internal");

        let is_entity = node
            .directives
            .iter()
            .any(|d| d.node.name.to_string() == "entity");

        if is_internal {
            self.parsed_graphql_schema
                .internal_types
                .insert(obj_name.clone());
        }

        if !is_entity && !is_internal {
            println!("Skipping TypeDefinition '{obj_name}', which is not marked with an @entity directive.");
            return;
        }
        self.parsed_graphql_schema
            .objects
            .insert(obj_name.clone(), o.clone());
        self.parsed_graphql_schema
            .parsed_typedef_names
            .insert(obj_name.clone());

        let is_virtual = node
            .directives
            .iter()
            .flat_map(|d| d.node.arguments.clone())
            .any(|t| t.0.node == "virtual" && t.1.node == ConstValue::Boolean(true));

        if is_virtual {
            self.parsed_graphql_schema
                .virtual_type_names
                .insert(obj_name.clone());

            GraphQLSchemaValidator::virtual_type_has_no_id_field(o, &obj_name);
        }

        // Since we have to use this manual `is_list_type` for each field, we might as well
        // keep track of how many m2m fields we have for this object here. We could also move this
        // logic to the `GraphQLSchemaValidator` itself, but that means we'd have to copy over the
        // `is_list_type` logic there as well.
        let mut m2m_field_count = 0;

        let mut field_mapping = BTreeMap::new();
        for (i, field) in o.fields.iter().enumerate() {
            GraphQLSchemaValidator::id_field_is_type_id(&field.node, &obj_name);

            let field_name = field.node.name.to_string();
            let field_typ_name = field.node.ty.to_string();
            let fid = field_id(&obj_name, &field_name);

            GraphQLSchemaValidator::ensure_fielddef_is_not_nested_list(&field.node);

            self.parsed_graphql_schema
                .object_ordered_fields
                .entry(obj_name.clone())
                .or_default()
                .push(OrderedField(field.node.clone(), i));

            // We need to add these field type names to `GraphQLSchemaValidator::list_field_types` prior to
            // doing the foreign key check below, (since we need to know whether a field is a FK type)
            if is_list_type(&field.node) {
                self.parsed_graphql_schema
                    .list_field_types
                    .insert(field_typ_name.replace('!', ""));

                self.parsed_graphql_schema
                    .list_type_defs
                    .insert(obj_name.clone(), node.clone());
            }

            // Manual foreign key check
            let ftype = field_type_name(&field.node);
            if self
                .parsed_graphql_schema
                .parsed_typedef_names
                .contains(&field_type_name(&field.node))
                && !self.parsed_graphql_schema.scalar_names.contains(&ftype)
                && !self.parsed_graphql_schema.enum_names.contains(&ftype)
                && !self
                    .parsed_graphql_schema
                    .virtual_type_names
                    .contains(&ftype)
                && !self.parsed_graphql_schema.internal_types.contains(&ftype)
                && !is_internal
            {
                GraphQLSchemaValidator::foreign_key_field_contains_no_unique_directive(
                    &field.node,
                    &obj_name,
                );

                let (ref_coltype, ref_colname, ref_tablename) = extract_foreign_key_info(
                    &field.node,
                    &self.parsed_graphql_schema.field_type_mappings,
                );

                if is_list_type(&field.node) {
                    GraphQLSchemaValidator::m2m_fk_field_ref_col_is_id(
                        &field.node,
                        &obj_name,
                        &ref_coltype,
                        &ref_colname,
                    );

                    m2m_field_count += 1;

                    GraphQLSchemaValidator::verify_m2m_relationship_count(
                        &obj_name,
                        m2m_field_count,
                    );

                    self.parsed_graphql_schema
                        .join_table_meta
                        .entry(obj_name.clone())
                        .or_default()
                        .push(JoinTableMeta::new(
                            &obj_name.to_lowercase(),
                            // The parent join column is _always_ `id: ID!`
                            IdCol::to_lowercase_str(),
                            &ref_tablename,
                            &ref_colname,
                            Some(i),
                        ));
                }

                let fk = self
                    .parsed_graphql_schema
                    .foreign_key_mappings
                    .get_mut(&obj_name.to_lowercase());
                match fk {
                    Some(fks_for_field) => {
                        fks_for_field.insert(
                            field.node.name.to_string(),
                            (
                                field_type_name(&field.node).to_lowercase(),
                                ref_colname.clone(),
                            ),
                        );
                    }
                    None => {
                        let fks_for_field = HashMap::from([(
                            field.node.name.to_string(),
                            (
                                field_type_name(&field.node).to_lowercase(),
                                ref_colname.clone(),
                            ),
                        )]);
                        self.parsed_graphql_schema
                            .foreign_key_mappings
                            .insert(obj_name.to_lowercase(), fks_for_field);
                    }
                }
            }

            let field_typ_name = field_type_name(&field.node);

            self.parsed_graphql_schema
                .parsed_typedef_names
                .insert(field_name.clone());
            field_mapping.insert(field_name, field_typ_name.clone());
            self.parsed_graphql_schema
                .field_type_optionality
                .insert(fid.clone(), field.node.ty.node.nullable);
            self.parsed_graphql_schema
                .field_type_mappings
                .insert(fid.clone(), field_typ_name);
            self.parsed_graphql_schema
                .field_defs
                .insert(fid, (field.node.clone(), obj_name.clone()));
        }

        self.parsed_graphql_schema
            .object_field_mappings
            .insert(obj_name, field_mapping);
    }

    fn build_typedef_names_to_types(&mut self) {
        self.parsed_graphql_schema.typedef_names_to_types = self
            .parsed_graphql_schema
            .type_defs
            .iter()
            .filter(|(_, t)| !matches!(&t.kind, TypeKind::Enum(_)))
            .collect::<Vec<(&String, &TypeDefinition)>>()
            .into_iter()
            .fold(HashMap::new(), |mut acc, (k, _)| {
                acc.insert(k.to_lowercase(), k.clone());
                acc
            });
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

type Account @entity {
    id: ID!
    address: Address!
    label: AccountLabel
}

type User @entity {
    id: ID!
    account: Account!
    username: String!
}

type Loser @entity {
    id: ID!
    account: Account!
    age: U64!
}

type Metadata @entity(virtual: true) {
    count: U64!
}

union Person = User | Loser


type Wallet @entity {
    id: ID!
    accounts: [Account!]!
}

type Safe @entity {
    id: ID!
    account: [Account!]!
}

type Vault @entity {
    id: ID!
    label: String!
    user: [User!]!
}

union Storage = Safe | Vault
"#;

        let parsed = ParsedGraphQLSchema::new(
            "test",
            "test",
            ExecutionSource::Wasm,
            Some(&GraphQLSchema::new(schema.to_string())),
        );

        assert!(parsed.is_ok());

        let parsed = parsed.unwrap();

        // Basic stuff
        assert!(parsed.has_type("Account"));
        assert!(parsed.has_type("User"));
        assert!(parsed.is_possible_foreign_key("Account"));
        assert!(parsed.is_virtual_typedef("Metadata"));
        assert!(parsed.is_enum_typedef("AccountLabel"));
        assert!(parsed
            .field_type_optionality()
            .contains_key("Account.label"));

        assert!(parsed.is_union_typedef("Person"));

        // Many to many for objects
        assert!(parsed.is_list_typedef("Wallet"));
        assert_eq!(parsed.join_table_meta().len(), 2);
        assert_eq!(
            parsed.join_table_meta().get("Wallet").unwrap()[0],
            JoinTableMeta::new("wallet", "id", "account", "id", Some(1))
        );

        // Many to many for unions
        assert!(!parsed.join_table_meta().contains_key("Safe"));
        assert!(!parsed.join_table_meta().contains_key("Vault"));
        assert!(parsed.join_table_meta().contains_key("Storage"));
        assert!(parsed.join_table_meta().get("Storage").unwrap().len() == 2);
        assert_eq!(
            parsed.join_table_meta().get("Storage").unwrap()[0],
            JoinTableMeta::new("storage", "id", "account", "id", Some(1))
        );
        assert_eq!(
            parsed.join_table_meta().get("Storage").unwrap()[1],
            JoinTableMeta::new("storage", "id", "user", "id", Some(3))
        );

        // Internal types
        assert!(parsed.internal_types.contains("AccountConnection"));
        assert!(parsed.internal_types.contains("AccountEdge"));
        assert!(parsed.internal_types.contains("UserConnection"));
        assert!(parsed.internal_types.contains("UserEdge"));
    }

    #[test]
    fn test_internal_type_defs_in_object_field_mapping() {
        let schema = r#"
type Foo @entity {
    id: ID!
    name: String!
}

type Bar @entity {
    id: ID!
    foo: [Foo!]!
}
"#;

        let parsed = ParsedGraphQLSchema::new(
            "test",
            "test",
            ExecutionSource::Wasm,
            Some(&GraphQLSchema::new(schema.to_string())),
        );

        assert!(parsed.is_ok());

        let parsed = parsed.unwrap();
        let bar_entity_fields = parsed.object_field_mappings.get("Bar").unwrap();
        assert_eq!(
            bar_entity_fields.get("fooConnection").unwrap(),
            &"FooConnection"
        );
    }

    /* Schema validation tests */
    #[test]
    #[should_panic(expected = "TypeDefinition name 'TransactionData' is reserved.")]
    fn test_schema_validator_check_disallowed_graphql_typedef_name() {
        let schema = r#"
type Foo @entity {
    id: ID!
}

type TransactionData @entity {
    id: ID!
}
"#;

        let _ = ParsedGraphQLSchema::new(
            "test",
            "test",
            ExecutionSource::Wasm,
            Some(&GraphQLSchema::new(schema.to_string())),
        )
        .unwrap();
    }

    #[test]
    #[should_panic(
        expected = "TypeDefinition(Union(Baz)) does not have consistent virtual/non-virtual members."
    )]
    fn test_schema_validator_check_derived_union_virtuality_is_well_formed() {
        let schema = r#"
type Foo @entity {
    id: ID!
    name: String!
}

type Bar @entity {
    id: ID!
    age: U64!
}

type Zoo @entity(virtual: true) {
    height: U64!
}

union Baz = Foo | Bar | Zoo
"#;

        let _ = ParsedGraphQLSchema::new(
            "test",
            "test",
            ExecutionSource::Wasm,
            Some(&GraphQLSchema::new(schema.to_string())),
        )
        .unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Derived type from Union(Baz) contains Field(name) which does not have a consistent type across all members."
    )]
    fn test_schema_validator_derived_field_type_is_consistent() {
        let schema = r#"
type Foo @entity {
    id: ID!
    name: String!
}

type Bar @entity {
    id: ID!
    age: U64!
}

type Zoo @entity {
    id: ID!
    name: U64!
}

union Baz = Foo | Bar | Zoo
"#;

        let _ = ParsedGraphQLSchema::new(
            "test",
            "test",
            ExecutionSource::Wasm,
            Some(&GraphQLSchema::new(schema.to_string())),
        )
        .unwrap();
    }

    #[test]
    #[should_panic(
        expected = "FieldDefinition(nested) is a nested list, which is not supported."
    )]
    fn test_schema_validator_ensure_fielddef_is_not_nested_list() {
        let schema = r#"
type Foo @entity {
    id: ID!
    name: String!
}

type Zoo @entity {
    id: ID!
    nested: [[Foo!]!]!
}
"#;

        let _ = ParsedGraphQLSchema::new(
            "test",
            "test",
            ExecutionSource::Wasm,
            Some(&GraphQLSchema::new(schema.to_string())),
        )
        .unwrap();
    }

    #[test]
    #[should_panic(
        expected = "FieldDefinition(id) on TypeDefinition(Foo) must be of type `ID!`. Found type `String!`."
    )]
    fn test_schema_validator_id_field_is_type_id() {
        let schema = r#"
type Foo @entity {
    id: String!
    name: String!
}"#;

        let _ = ParsedGraphQLSchema::new(
            "test",
            "test",
            ExecutionSource::Wasm,
            Some(&GraphQLSchema::new(schema.to_string())),
        )
        .unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Virtual TypeDefinition(Foo) cannot contain an `id: ID!` FieldDefinition."
    )]
    fn test_schema_validator_virtual_type_has_no_id_field() {
        let schema = r#"
type Foo @entity(virtual: true) {
    id: ID!
    name: String!
}"#;

        let _ = ParsedGraphQLSchema::new(
            "test",
            "test",
            ExecutionSource::Wasm,
            Some(&GraphQLSchema::new(schema.to_string())),
        )
        .unwrap();
    }

    #[test]
    #[should_panic(
        expected = "FieldDefinition(id) on TypeDefinition(Bar) must be of type `ID!`. Found type `ID`."
    )]
    fn test_schema_validator_foreign_key_field_contains_no_unique_directive() {
        let schema = r#"
type Foo @entity {
    id: ID!
    name: String!
}

type Bar @entity {
    id: ID
    foo: Foo! @unique
}
"#;

        let _ = ParsedGraphQLSchema::new(
            "test",
            "test",
            ExecutionSource::Wasm,
            Some(&GraphQLSchema::new(schema.to_string())),
        )
        .unwrap();
    }

    #[test]
    #[should_panic(
        expected = "TypeDefinition(Bar) exceeds the allowed number of many-to-many` relationships. The maximum allowed is 10."
    )]
    fn test_schema_validator_verify_m2m_relationship_count() {
        let schema = r#"
type Type1 @entity {
    id: ID!
    name: String!
}

type Type2 @entity {
    id: ID!
    name: String!
}

type Type3 @entity {
    id: ID!
    name: String!
}

type Type4 @entity {
    id: ID!
    name: String!
}

type Type5 @entity {
    id: ID!
    name: String!
}

type Type6 @entity {
    id: ID!
    name: String!
}

type Type7 @entity {
    id: ID!
    name: String!
}

type Type8 @entity {
    id: ID!
    name: String!
}

type Type9 @entity {
    id: ID!
    name: String!
}

type Type10 @entity {
    id: ID!
    name: String!
}

type Type11 @entity {
    id: ID!
    name: String!
}

type Bar @entity {
    id: ID!
    type1: [Type1!]!
    type2: [Type2!]!
    type3: [Type3!]!
    type4: [Type4!]!
    type5: [Type5!]!
    type6: [Type6!]!
    type7: [Type7!]!
    type8: [Type8!]!
    type9: [Type9!]!
    type10: [Type10!]!
    type11: [Type11!]!
}
"#;

        let _ = ParsedGraphQLSchema::new(
            "test",
            "test",
            ExecutionSource::Wasm,
            Some(&GraphQLSchema::new(schema.to_string())),
        )
        .unwrap();
    }

    #[test]
    #[should_panic(
        expected = "FieldDefinition(foo) on TypeDefinition(Bar) is a many-to-many relationship where the inner scalar is of type `name: String!`. However, only inner scalars of type `id: ID!` are allowed."
    )]
    fn test_schema_validator_m2m_fk_field_ref_col_is_id() {
        let schema = r#"
type Foo @entity {
    id: ID!
    name: String!
}

type Bar @entity {
    id: ID!
    foo: [Foo!]! @join(on:name)
}
"#;

        let _ = ParsedGraphQLSchema::new(
            "test",
            "test",
            ExecutionSource::Wasm,
            Some(&GraphQLSchema::new(schema.to_string())),
        )
        .unwrap();
    }
}

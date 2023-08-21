//! # fuel_indexer_lib::parser
//!
//! A utility used to help parse and cache various components of indexer
//! GraphQL schema. This is meant to be a productivity tool for project devs.

use crate::{
    fully_qualified_namespace,
    graphql::{
        extract_foreign_key_info, field_id, field_type_name, is_list_type,
        list_field_type_name, GraphQLSchema, GraphQLSchemaValidator, IdCol, BASE_SCHEMA,
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

    /// The parsed schema AST.
    ast: ServiceDocument,

    /// Mapping of fully qualified field names to their `FieldDefinition` and `TypeDefinition` name.
    ///
    /// We keep the `TypeDefinition` name so that we can know what type of object the field belongs to.
    field_defs: HashMap<String, (FieldDefinition, String)>,

    /// Raw GraphQL schema content.
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
            ast,
            schema: GraphQLSchema::default(),
            list_field_types: HashSet::new(),
            list_type_defs: HashMap::new(),
            unions: HashMap::new(),
            join_table_meta: HashMap::new(),
            object_ordered_fields: HashMap::new(),
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
        let mut decoder = SchemaDecoder::new();

        if let Some(schema) = schema {
            // Parse _everything_ in the GraphQL schema
            let ast = parse_schema(schema.schema())?;
            decoder.decode_service_document(ast)?;

            decoder.parsed_graphql_schema.namespace = namespace.to_string();
            decoder.parsed_graphql_schema.identifier = identifier.to_string();
            decoder.parsed_graphql_schema.exec_source = exec_source.clone();
            decoder.parsed_graphql_schema.schema = schema.clone();
        };

        let mut result = decoder.get_parsed_schema();

        let base_type_names = {
            let base_ast = parse_schema(BASE_SCHEMA)?;
            let mut base_decoder = SchemaDecoder::new();
            base_decoder.decode_service_document(base_ast)?;
            base_decoder.parsed_graphql_schema.type_names
        };

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

    /// The parsed schema AST.
    pub fn ast(&self) -> &ServiceDocument {
        &self.ast
    }

    /// Raw GraphQL schema content.
    pub fn schema(&self) -> &GraphQLSchema {
        &self.schema
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
                return "Virtual".to_string();
            } else if self.is_enum_typedef(&typ_name) {
                return "Charfield".to_string();
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
            return "Virtual".to_string();
        }

        if self.is_enum_typedef(&typ_name) {
            return "Charfield".to_string();
        }

        typ_name
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
        self.parsed_typedef_names.contains(name)
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

    // Decoder functions to iteratively build up the ParsedGraphQLSchema struct.
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
        self.parsed_graphql_schema.ast = ast;
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
        self.parsed_graphql_schema
            .parsed_typedef_names
            .insert(union_name.clone());
        self.parsed_graphql_schema
            .unions
            .insert(union_name.clone(), node.clone());

        self.parsed_graphql_schema
            .union_names
            .insert(union_name.clone());

        GraphQLSchemaValidator::check_derived_union_is_well_formed(
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

                if processed_fields.contains(&field_id) {
                    return;
                }

                processed_fields.insert(field_id);

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
            let member_obj = self
                .parsed_graphql_schema
                .objects
                .get(&member_name)
                .unwrap();
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
        // Only parse `TypeDefinition`s with the `@entity` directive.
        let is_entity = node
            .directives
            .iter()
            .any(|d| d.node.name.to_string() == "entity");

        if !is_entity {
            //continue;
            return;
        }
        self.parsed_graphql_schema
            .objects
            .insert(obj_name.clone(), o.clone());
        self.parsed_graphql_schema
            .parsed_typedef_names
            .insert(obj_name.clone());

        let mut field_mapping = BTreeMap::new();
        for (i, field) in o.fields.iter().enumerate() {
            let field_name = field.node.name.to_string();
            let field_typ_name = field.node.ty.to_string();
            let fid = field_id(&obj_name, &field_name);

            self.parsed_graphql_schema
                .object_ordered_fields
                .entry(obj_name.clone())
                .or_default()
                .push(OrderedField(field.node.clone(), i));

            if is_list_type(&field.node) {
                self.parsed_graphql_schema
                    .list_field_types
                    .insert(field_typ_name.replace('!', ""));

                self.parsed_graphql_schema
                    .list_type_defs
                    .insert(obj_name.clone(), node.clone());
            }

            let is_virtual = node
                .directives
                .iter()
                .flat_map(|d| d.node.arguments.clone())
                .any(|t| t.0.node == "virtual");

            if is_virtual {
                self.parsed_graphql_schema
                    .virtual_type_names
                    .insert(obj_name.clone());
            }

            // Manual version of `ParsedGraphQLSchema::is_possible_foreign_key`
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
            {
                let (_ref_coltype, ref_colname, ref_tablename) = extract_foreign_key_info(
                    &field.node,
                    &self.parsed_graphql_schema.field_type_mappings,
                );

                if is_list_type(&field.node) {
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

fn assert_schema_eq(result: &ParsedGraphQLSchema, old_result: &ParsedGraphQLSchema) {
    assert_eq!(result.type_names, old_result.type_names, "type_names");
    assert_eq!(
        result.typedef_names_to_types, old_result.typedef_names_to_types,
        "typedef_names_to_types"
    );

    let mut x = result.objects.keys().collect::<Vec<&String>>();
    x.sort();
    let mut y = old_result.objects.keys().collect::<Vec<&String>>();
    y.sort();

    assert_eq!(x, y, "objects");

    let mut x = result.unions.keys().collect::<Vec<&String>>();
    x.sort();
    let mut y = old_result.unions.keys().collect::<Vec<&String>>();
    y.sort();
    assert_eq!(x, y, "unions");

    assert_eq!(result.enum_names, old_result.enum_names, "enum_names");

    assert_eq!(result.union_names, old_result.union_names, "union_names");

    assert_eq!(
        result.object_field_mappings, old_result.object_field_mappings,
        "object_field_mappings"
    );

    assert_eq!(
        result.virtual_type_names, old_result.virtual_type_names,
        "virtual_type_names"
    );

    assert_eq!(
        result.parsed_typedef_names, old_result.parsed_typedef_names,
        "parsed_typedef_names"
    );

    assert_eq!(
        result.field_type_mappings, old_result.field_type_mappings,
        "field_type_mappings"
    );

    assert_eq!(result.scalar_names, old_result.scalar_names, "scalar_names");

    assert_eq!(
        result.field_type_optionality, old_result.field_type_optionality,
        "field_type_optionality"
    );

    let mut x = result.field_defs.keys().collect::<Vec<&String>>();
    x.sort();
    let mut y = old_result.field_defs.keys().collect::<Vec<&String>>();
    y.sort();
    assert_eq!(x, y, "field_defs");

    assert_eq!(
        result.foreign_key_mappings, old_result.foreign_key_mappings,
        "foreign_key_mappings"
    );

    let mut x = result.type_defs.keys().collect::<Vec<&String>>();
    x.sort();
    let mut y = old_result.type_defs.keys().collect::<Vec<&String>>();
    y.sort();
    assert_eq!(x, y, "type_defs");

    assert_eq!(
        result.list_field_types, old_result.list_field_types,
        "list_field_types"
    );

    let mut x = result.list_type_defs.keys().collect::<Vec<&String>>();
    x.sort();
    let mut y = old_result.list_type_defs.keys().collect::<Vec<&String>>();
    y.sort();
    assert_eq!(x, y, "list_type_defs");

    assert_eq!(
        result.join_table_meta, old_result.join_table_meta,
        "join_table_meta"
    );

    let mut x = result
        .object_ordered_fields
        .keys()
        .collect::<Vec<&String>>();
    x.sort();
    let mut y = old_result
        .object_ordered_fields
        .keys()
        .collect::<Vec<&String>>();
    y.sort();
    assert_eq!(x, y, "object_ordered_fields");

    assert_eq!(result.schema.schema(), old_result.schema.schema(), "schema");
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
    username: Charfield!
}

type Loser @entity {
    id: ID!
    account: Account!
    age: UInt8!
}

type Metadata @entity(virtual: true) {
    count: UInt8!
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
    label: Charfield!
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
    }
}

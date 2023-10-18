pub mod constants;
pub mod parser;
pub mod schema_gen;
pub mod types;
pub mod validator;

use async_graphql_value::Name;
pub use parser::{JoinTableMeta, ParsedError, ParsedGraphQLSchema};
pub use validator::GraphQLSchemaValidator;

use async_graphql_parser::{
    types::{
        BaseType, ConstDirective, FieldDefinition, ObjectType, ServiceDocument, Type,
        TypeDefinition, TypeKind, TypeSystemDefinition,
    },
    Pos, Positioned,
};
use fuel_indexer_types::graphql::IndexMetadata;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use types::IdCol;

/// Maximum amount of foreign key list fields that can exist on a `TypeDefinition`
pub const MAX_FOREIGN_KEY_LIST_FIELDS: usize = 10;

/// Base GraphQL schema containing base scalars.
pub const BASE_SCHEMA: &str = include_str!("./base.graphql");

/// Derive version of GraphQL schema content via SHA256.
pub fn schema_version(schema: &str) -> String {
    format!("{:x}", Sha256::digest(schema.as_bytes()))
}

/// Inject native entities into the GraphQL schema.
fn inject_native_entities_into_schema(schema: &str) -> String {
    if !schema.contains("type IndexMetadataEntity") {
        format!("{}{}", schema, IndexMetadata::schema_fragment())
    } else {
        schema.to_string()
    }
}

/// Inject internal types into the schema. In order to support popular
/// functionality (e.g. cursor-based pagination) and minimize the amount
/// of types that a user needs to create, internal types are injected into
/// the `ServiceDocument`. These types are not used to create database tables/columns
/// or entity structs in handler functions.
pub(crate) fn inject_internal_types_into_document(
    mut ast: ServiceDocument,
    base_type_names: &HashSet<String>,
) -> ServiceDocument {
    let mut pagination_types: Vec<TypeSystemDefinition> = Vec::new();
    pagination_types.push(create_page_info_type_def());

    // Iterate through all objects in document and create special
    // pagination types for each object with a list field.
    for ty in ast.definitions.iter_mut() {
        if let TypeSystemDefinition::Type(t) = ty {
            if let TypeKind::Object(obj) = &mut t.node.kind {
                let mut internal_fields: Vec<Positioned<FieldDefinition>> = Vec::new();

                for f in &obj.fields {
                    if let BaseType::List(inner_type) = &f.node.ty.node.base {
                        if let BaseType::Named(name) = &inner_type.base {
                            if base_type_names.contains(&name.to_string()) {
                                continue;
                            }
                            let edge_type = create_edge_type_for_list_field(f);
                            pagination_types.push(edge_type);

                            let connection_type =
                                create_connection_type_def_for_list_entity(name);
                            pagination_types.push(connection_type);

                            let connection_field = Positioned::position_node(
                                f,
                                FieldDefinition {
                                    description: None,
                                    name: Positioned::position_node(
                                        f,
                                        Name::new(format!(
                                            "{}Connection",
                                            f.node.name.node
                                        )),
                                    ),
                                    arguments: vec![],
                                    ty: Positioned::position_node(
                                        f,
                                        Type {
                                            base: BaseType::Named(Name::new(format!(
                                                "{name}Connection"
                                            ))),
                                            nullable: false,
                                        },
                                    ),
                                    directives: vec![Positioned::position_node(
                                        f,
                                        ConstDirective {
                                            name: Positioned::position_node(
                                                f,
                                                Name::new("internal"),
                                            ),
                                            arguments: vec![],
                                        },
                                    )],
                                },
                            );
                            internal_fields.push(connection_field);
                        }
                    }
                }

                obj.fields.append(&mut internal_fields);
            }
        }
    }

    ast.definitions.append(&mut pagination_types);

    ast
}

fn create_edge_type_for_list_field(
    list_field: &Positioned<FieldDefinition>,
) -> TypeSystemDefinition {
    let (base_type, name) = if let BaseType::List(t) = &list_field.node.ty.node.base {
        if let BaseType::Named(n) = &t.base {
            (t, n)
        } else {
            unreachable!("Edge type creation should not occur for non-list fields")
        }
    } else {
        unreachable!("Edge type creation should not occur for non-list fields")
    };

    let edge_obj_type = ObjectType {
        implements: vec![],
        fields: vec![
            Positioned::position_node(
                list_field,
                FieldDefinition {
                    description: None,
                    name: Positioned::position_node(list_field, Name::new("node")),
                    arguments: vec![],
                    ty: Positioned::position_node(
                        list_field,
                        Type {
                            base: base_type.base.clone(),
                            nullable: false,
                        },
                    ),
                    directives: vec![],
                },
            ),
            Positioned::position_node(
                list_field,
                FieldDefinition {
                    description: None,
                    name: Positioned::position_node(list_field, Name::new("cursor")),
                    arguments: vec![],
                    ty: Positioned::position_node(
                        list_field,
                        Type {
                            base: BaseType::Named(Name::new("String")),
                            nullable: false,
                        },
                    ),
                    directives: vec![],
                },
            ),
        ],
    };

    TypeSystemDefinition::Type(Positioned::position_node(
        list_field,
        TypeDefinition {
            extend: false,
            description: None,
            name: Positioned::position_node(
                list_field,
                Name::new(format!("{}Edge", name)),
            ),
            directives: vec![Positioned::position_node(
                list_field,
                ConstDirective {
                    name: Positioned::position_node(list_field, Name::new("internal")),
                    arguments: vec![],
                },
            )],
            kind: TypeKind::Object(edge_obj_type),
        },
    ))
}

/// Generate connection type defintion for a list field on an entity.
fn create_connection_type_def_for_list_entity(name: &Name) -> TypeSystemDefinition {
    let dummy_position = Pos {
        line: usize::MAX,
        column: usize::MAX,
    };

    let obj_type = ObjectType {
        implements: vec![],
        fields: vec![
            Positioned::new(
                FieldDefinition {
                    description: None,
                    name: Positioned::new(Name::new("nodes"), dummy_position),
                    arguments: vec![],
                    ty: Positioned::new(
                        Type {
                            base: BaseType::List(Box::new(Type {
                                base: BaseType::Named(name.clone()),
                                nullable: false,
                            })),
                            nullable: false,
                        },
                        dummy_position,
                    ),
                    directives: vec![],
                },
                dummy_position,
            ),
            Positioned::new(
                FieldDefinition {
                    description: None,
                    name: Positioned::new(Name::new("edges"), dummy_position),
                    arguments: vec![],
                    ty: Positioned::new(
                        Type {
                            base: BaseType::List(Box::new(Type {
                                base: BaseType::Named(Name::new(format!(
                                    "{}Edge",
                                    name.clone()
                                ))),
                                nullable: false,
                            })),
                            nullable: false,
                        },
                        dummy_position,
                    ),
                    directives: vec![],
                },
                dummy_position,
            ),
            Positioned::new(
                FieldDefinition {
                    description: None,
                    name: Positioned::new(Name::new("pageInfo"), dummy_position),
                    arguments: vec![],
                    ty: Positioned::new(
                        Type {
                            base: BaseType::Named(Name::new("PageInfo")),
                            nullable: false,
                        },
                        dummy_position,
                    ),
                    directives: vec![],
                },
                dummy_position,
            ),
        ],
    };

    TypeSystemDefinition::Type(Positioned::new(
        TypeDefinition {
            extend: false,
            description: None,
            name: Positioned::new(
                Name::new(format!("{}Connection", name.clone())),
                dummy_position,
            ),
            directives: vec![Positioned::new(
                ConstDirective {
                    name: Positioned::new(Name::new("internal"), dummy_position),
                    arguments: vec![],
                },
                dummy_position,
            )],
            kind: TypeKind::Object(obj_type),
        },
        dummy_position,
    ))
}

/// Generate `PageInfo` type defintion for use in connection type defintions.
fn create_page_info_type_def() -> TypeSystemDefinition {
    let dummy_position = Pos {
        line: usize::MAX,
        column: usize::MAX,
    };

    let obj_type = ObjectType {
        implements: vec![],
        fields: vec![
            Positioned::new(
                FieldDefinition {
                    description: None,
                    name: Positioned::new(Name::new("hasPreviousPage"), dummy_position),
                    arguments: vec![],
                    ty: Positioned::new(
                        Type {
                            base: BaseType::Named(Name::new("Boolean")),
                            nullable: false,
                        },
                        dummy_position,
                    ),
                    directives: vec![],
                },
                dummy_position,
            ),
            Positioned::new(
                FieldDefinition {
                    description: None,
                    name: Positioned::new(Name::new("hasNextPage"), dummy_position),
                    arguments: vec![],
                    ty: Positioned::new(
                        Type {
                            base: BaseType::Named(Name::new("Boolean")),
                            nullable: false,
                        },
                        dummy_position,
                    ),
                    directives: vec![],
                },
                dummy_position,
            ),
            Positioned::new(
                FieldDefinition {
                    description: None,
                    name: Positioned::new(Name::new("startCursor"), dummy_position),
                    arguments: vec![],
                    ty: Positioned::new(
                        Type {
                            base: BaseType::Named(Name::new("String")),
                            nullable: true,
                        },
                        dummy_position,
                    ),
                    directives: vec![],
                },
                dummy_position,
            ),
            Positioned::new(
                FieldDefinition {
                    description: None,
                    name: Positioned::new(Name::new("endCursor"), dummy_position),
                    arguments: vec![],
                    ty: Positioned::new(
                        Type {
                            base: BaseType::Named(Name::new("String")),
                            nullable: true,
                        },
                        dummy_position,
                    ),
                    directives: vec![],
                },
                dummy_position,
            ),
            Positioned::new(
                FieldDefinition {
                    description: None,
                    name: Positioned::new(Name::new("totalCount"), dummy_position),
                    arguments: vec![],
                    ty: Positioned::new(
                        Type {
                            base: BaseType::Named(Name::new("U64")),
                            nullable: false,
                        },
                        dummy_position,
                    ),
                    directives: vec![],
                },
                dummy_position,
            ),
        ],
    };

    TypeSystemDefinition::Type(Positioned::new(
        TypeDefinition {
            extend: false,
            description: None,
            name: Positioned::new(Name::new("PageInfo"), dummy_position),
            directives: vec![Positioned::new(
                ConstDirective {
                    name: Positioned::new(Name::new("internal"), dummy_position),
                    arguments: vec![],
                },
                dummy_position,
            )],
            kind: TypeKind::Object(obj_type),
        },
        dummy_position,
    ))
}

pub fn check_for_directive(
    directives: &[Positioned<ConstDirective>],
    directive_name: &str,
) -> bool {
    directives
        .iter()
        .any(|d| d.node.name.node == directive_name)
}

/// Wrapper for GraphQL schema content.
#[derive(Default, Debug, Clone)]
pub struct GraphQLSchema {
    /// Raw GraphQL schema content.
    schema: String,

    /// Version of the schema.
    version: String,
}

impl From<String> for GraphQLSchema {
    fn from(s: String) -> Self {
        let schema = inject_native_entities_into_schema(&s);
        let version = schema_version(&s);
        Self { schema, version }
    }
}

impl GraphQLSchema {
    /// Create a new `GraphQLSchema` from raw GraphQL content.
    pub fn new(content: String) -> Self {
        let schema = inject_native_entities_into_schema(&content);
        let version = schema_version(&schema);
        Self { schema, version }
    }

    pub fn schema(&self) -> &str {
        &self.schema
    }

    pub fn version(&self) -> &str {
        &self.version
    }
}

impl From<&GraphQLSchema> for Vec<u8> {
    fn from(schema: &GraphQLSchema) -> Self {
        schema.schema().as_bytes().to_vec()
    }
}

impl From<Vec<u8>> for GraphQLSchema {
    fn from(value: Vec<u8>) -> Self {
        GraphQLSchema::new(String::from_utf8_lossy(&value).to_string())
    }
}

impl std::fmt::Display for GraphQLSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.schema)
    }
}

/// Given a `FieldDefinition` that is a possible foreign key (according to `ParsedGraphQLSchema`),
/// return the column type, column name, and table name of the foreign key.

// We pass `ParsedGraphQLSchema::field_type_mappings` here instead of the full `ParsedGraphQLSchema`
// because when using `extract_foreign_key_info` in `ParsedGraphQLSchema` we don't have access to the
// fully parsed `ParsedGraphQLSchema` yet.
pub fn extract_foreign_key_info(
    f: &FieldDefinition,
    field_type_mappings: &HashMap<String, String>,
) -> (String, String, String) {
    let (ref_coltype, ref_colname, ref_tablename) = f
        .directives
        .iter()
        .find(|d| d.node.name.to_string() == "join")
        .map(|d| {
            let typdef_name = field_type_name(f);
            let ref_field_name = d
                .clone()
                .node
                .arguments
                .pop()
                .expect("Expected directive info")
                .1
                .to_string();
            let fk_fid = field_id(&typdef_name, &ref_field_name);
            let fk_field_type = field_type_mappings
                .get(&fk_fid)
                .expect("Field ID not found in schema")
                .to_string();

            (
                fk_field_type.replace(['[', ']', '!'], ""),
                ref_field_name,
                typdef_name.to_lowercase(),
            )
        })
        .unwrap_or((
            "UID".to_string(),
            IdCol::to_lowercase_string(),
            field_type_name(f).to_lowercase(),
        ));

    (ref_coltype, ref_colname, ref_tablename)
}

/// Return a fully qualified name for a given `FieldDefinition` on a given `TypeDefinition`.
pub fn field_id(typdef_name: &str, field_name: &str) -> String {
    format!("{typdef_name}.{field_name}")
}

/// Whether a given `FieldDefinition` is a `List` type.
pub fn is_list_type(f: &FieldDefinition) -> bool {
    f.ty.to_string().matches(['[', ']']).count() == 2
}

/// Return the simple field name for a given `FieldDefinition`.
pub fn field_type_name(f: &FieldDefinition) -> String {
    f.ty.to_string().replace(['[', ']', '!'], "")
}

/// Return the simple field name for a given list `FieldDefinition`.
pub fn list_field_type_name(f: &FieldDefinition) -> String {
    f.ty.to_string().replace(['!'], "")
}

pub mod constants;
pub mod parser;
pub mod types;
pub mod validator;

use async_graphql_value::Name;
pub use parser::{JoinTableMeta, ParsedError, ParsedGraphQLSchema};
pub use validator::GraphQLSchemaValidator;

use async_graphql_parser::{
    types::{
        BaseType, ConstDirective, EnumType, EnumValueDefinition, FieldDefinition,
        InputObjectType, InputValueDefinition, ObjectType, SchemaDefinition,
        ServiceDocument, Type, TypeDefinition, TypeKind, TypeSystemDefinition,
    },
    Pos, Positioned,
};
use fuel_indexer_types::graphql::IndexMetadata;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use types::IdCol;

lazy_static::lazy_static!(
    static ref SORTABLE_SCALAR_TYPES: HashSet<&'static str> = HashSet::from([
        "Address",
        "AssetId",
        "ContractId",
        "I128",
        "I16",
        "I32",
        "I64",
        "ID",
        "Identity",
        "String",
        "U128",
        "U16",
        "U32",
        "U64",
        "UID",
    ]);
);

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

pub(crate) fn inject_query_type(
    mut ast: ServiceDocument,
    input_obj_type_def_map: HashMap<
        String,
        (Option<TypeSystemDefinition>, Option<TypeSystemDefinition>),
    >,
) -> ServiceDocument {
    let dummy_position = Pos {
        line: usize::MAX,
        column: usize::MAX,
    };

    let mut fields: Vec<Positioned<FieldDefinition>> = vec![];
    for ty in ast.definitions.iter() {
        if let TypeSystemDefinition::Type(t) = ty {
            if t.node.name.node == "IndexMetadataEntity" {
                continue;
            }
            if let TypeKind::Object(obj) = &t.node.kind {
                if check_for_directive(&t.node.directives, "queryable") {
                    if !check_for_directive(&t.node.directives, "internal") {
                        // We are now parsing a user-generated entity.
                        let selector = obj.fields.iter().find(|f| {
                            check_for_directive(&f.node.directives, "search")
                                && check_for_directive(&f.node.directives, "unique")
                        });
                        if let Some(selector_field) = selector {
                            let field = Positioned::new(
                                FieldDefinition {
                                    description: None,
                                    name: Positioned::new(
                                        Name::new(t.node.name.node.to_lowercase()),
                                        dummy_position,
                                    ),
                                    arguments: vec![Positioned::new(
                                        InputValueDefinition {
                                            description: None,
                                            name: selector_field.node.name.clone(),
                                            ty: selector_field.node.ty.clone(),
                                            default_value: None,
                                            directives: vec![],
                                        },
                                        dummy_position,
                                    )],
                                    ty: Positioned::new(
                                        Type {
                                            base: BaseType::Named(
                                                t.node.name.node.clone(),
                                            ),
                                            nullable: true,
                                        },
                                        dummy_position,
                                    ),
                                    directives: vec![],
                                },
                                dummy_position,
                            );
                            fields.push(field);
                        } else {
                            // change signature to return error
                        }
                    } else if check_for_directive(&t.node.directives, "internal")
                        && check_for_directive(&t.node.directives, "connection")
                    {
                        let mut field = Positioned::new(
                            FieldDefinition {
                                description: None,
                                name: Positioned::new(
                                    Name::new(format!(
                                        "{}s",
                                        t.node
                                            .name
                                            .node
                                            .replace("Connection", "")
                                            .to_lowercase()
                                    )),
                                    dummy_position,
                                ),
                                arguments: vec![
                                    Positioned::new(
                                        InputValueDefinition {
                                            description: None,
                                            name: Positioned::new(
                                                Name::new("first"),
                                                dummy_position,
                                            ),
                                            ty: Positioned::new(
                                                Type {
                                                    base: BaseType::Named(Name::new(
                                                        "U64",
                                                    )),
                                                    nullable: true,
                                                },
                                                dummy_position,
                                            ),
                                            default_value: None,
                                            directives: vec![],
                                        },
                                        dummy_position,
                                    ),
                                    Positioned::new(
                                        InputValueDefinition {
                                            description: None,
                                            name: Positioned::new(
                                                Name::new("after"),
                                                dummy_position,
                                            ),
                                            ty: Positioned::new(
                                                Type {
                                                    base: BaseType::Named(Name::new(
                                                        "String",
                                                    )),
                                                    nullable: true,
                                                },
                                                dummy_position,
                                            ),
                                            default_value: None,
                                            directives: vec![],
                                        },
                                        dummy_position,
                                    ),
                                    Positioned::new(
                                        InputValueDefinition {
                                            description: None,
                                            name: Positioned::new(
                                                Name::new("last"),
                                                dummy_position,
                                            ),
                                            ty: Positioned::new(
                                                Type {
                                                    base: BaseType::Named(Name::new(
                                                        "U64",
                                                    )),
                                                    nullable: true,
                                                },
                                                dummy_position,
                                            ),
                                            default_value: None,
                                            directives: vec![],
                                        },
                                        dummy_position,
                                    ),
                                    Positioned::new(
                                        InputValueDefinition {
                                            description: None,
                                            name: Positioned::new(
                                                Name::new("before"),
                                                dummy_position,
                                            ),
                                            ty: Positioned::new(
                                                Type {
                                                    base: BaseType::Named(Name::new(
                                                        "String",
                                                    )),
                                                    nullable: true,
                                                },
                                                dummy_position,
                                            ),
                                            default_value: None,
                                            directives: vec![],
                                        },
                                        dummy_position,
                                    ),
                                ],
                                ty: Positioned::new(
                                    Type {
                                        base: BaseType::Named(t.node.name.node.clone()),
                                        nullable: false,
                                    },
                                    dummy_position,
                                ),
                                directives: vec![],
                            },
                            dummy_position,
                        );

                        if let Some((sort_input_obj, filter_input_obj)) =
                            input_obj_type_def_map
                                .get(&t.node.name.node.replace("Connection", ""))
                        {
                            if sort_input_obj.is_some() {
                                field.node.arguments.push(Positioned::new(
                                    InputValueDefinition {
                                        description: None,
                                        name: Positioned::new(
                                            Name::new("order"),
                                            dummy_position,
                                        ),
                                        ty: Positioned::new(
                                            Type {
                                                base: BaseType::Named(Name::new(
                                                    format!(
                                                        "{}OrderInput",
                                                        t.node
                                                            .name
                                                            .node
                                                            .replace("Connection", "")
                                                    ),
                                                )),
                                                nullable: false,
                                            },
                                            dummy_position,
                                        ),
                                        default_value: None,
                                        directives: vec![],
                                    },
                                    dummy_position,
                                ))
                            }

                            if filter_input_obj.is_some() {
                                field.node.arguments.push(Positioned::new(
                                    InputValueDefinition {
                                        description: None,
                                        name: Positioned::new(
                                            Name::new("filter"),
                                            dummy_position,
                                        ),
                                        ty: Positioned::new(
                                            Type {
                                                base: BaseType::Named(Name::new(
                                                    format!(
                                                        "{}FilterInput",
                                                        t.node
                                                            .name
                                                            .node
                                                            .replace("Connection", "")
                                                    ),
                                                )),
                                                nullable: true,
                                            },
                                            dummy_position,
                                        ),
                                        default_value: None,
                                        directives: vec![],
                                    },
                                    dummy_position,
                                ))
                            }
                        }

                        fields.push(field);
                    }
                }
            }
        }
    }

    let query_type_def = TypeSystemDefinition::Type(Positioned::new(
        TypeDefinition {
            extend: false,
            description: None,
            name: Positioned::new(Name::new("Query"), dummy_position),
            directives: vec![Positioned::new(
                ConstDirective {
                    name: Positioned::new(Name::new("internal"), dummy_position),
                    arguments: vec![],
                },
                dummy_position,
            )],
            kind: TypeKind::Object(ObjectType {
                implements: vec![],
                fields,
            }),
        },
        dummy_position,
    ));

    let schema_def = TypeSystemDefinition::Schema(Positioned::new(
        SchemaDefinition {
            extend: false,
            directives: vec![Positioned::new(
                ConstDirective {
                    name: Positioned::new(Name::new("internal"), dummy_position),
                    arguments: vec![],
                },
                dummy_position,
            )],
            query: Some(Positioned::new(Name::new("Query"), dummy_position)),
            mutation: None,
            subscription: None,
        },
        dummy_position,
    ));

    ast.definitions.push(query_type_def);
    ast.definitions.push(schema_def);

    ast
}

/// Inject internal types into the schema. In order to support popular
/// functionality (e.g. cursor-based pagination) and minimize the amount
/// of types that a user needs to create, internal types are injected into
/// the `ServiceDocument`. These types are not used to create database tables/columns
/// or entity structs in handler functions.
pub(crate) fn inject_internal_types_into_document(
    mut ast: ServiceDocument,
) -> ServiceDocument {
    ast.definitions.push(create_sort_order_enum());
    ast.definitions.push(create_comparison_obj_for_filtering());

    let input_obj_type_def_map = create_input_object_types(&ast);
    for (_, (sort_input_obj, filter_input_obj)) in input_obj_type_def_map.iter() {
        if let Some(s_obj) = sort_input_obj {
            ast.definitions.push(s_obj.clone());
        }
        if let Some(f_obj) = filter_input_obj {
            ast.definitions.push(f_obj.clone());
        }
    }

    ast.definitions.append(&mut create_pagination_types(&ast));

    ast = inject_query_type(ast, input_obj_type_def_map);

    ast
}

fn create_input_object_types(
    ast: &ServiceDocument,
) -> HashMap<String, (Option<TypeSystemDefinition>, Option<TypeSystemDefinition>)> {
    let mut input_obj_type_def_map: HashMap<
        String,
        (Option<TypeSystemDefinition>, Option<TypeSystemDefinition>),
    > = HashMap::new();

    // Iterate through all objects in document and create special
    // pagination types for each object with a list field.
    for ty in ast.definitions.iter() {
        if let TypeSystemDefinition::Type(t) = ty {
            if t.node.name.node == "IndexMetadataEntity" {
                continue;
            }

            if let TypeKind::Object(obj) = &t.node.kind {
                input_obj_type_def_map.insert(
                    t.node.name.node.to_string(),
                    (
                        create_sort_order_input_obj(obj, &t.node.name.node),
                        create_filter_input_obj_for_entity(obj, &t.node.name.node),
                    ),
                );
            }
        }
    }

    input_obj_type_def_map
}

fn create_pagination_types(ast: &ServiceDocument) -> Vec<TypeSystemDefinition> {
    let mut pagination_types: Vec<TypeSystemDefinition> = Vec::new();
    pagination_types.push(create_page_info_type_def());

    // Iterate through all objects in document and create special
    // pagination types for each object with a list field.
    for ty in ast.definitions.iter() {
        if let TypeSystemDefinition::Type(t) = ty {
            if t.node.name.node == "IndexMetadataEntity" {
                continue;
            }

            if matches!(&t.node.kind, TypeKind::Object(_)) {
                let edge_type = create_edge_type(t);
                pagination_types.push(edge_type);

                let connection_type = create_connection_type_def(
                    &t.node.name.node,
                    check_for_directive(&t.node.directives, "queryable"),
                );
                pagination_types.push(connection_type);
            }
        }
    }

    pagination_types
}

fn create_sort_order_enum() -> TypeSystemDefinition {
    let dummy_position = Pos {
        line: usize::MAX,
        column: usize::MAX,
    };
    let sort_enum_type = EnumType {
        values: vec![
            Positioned::new(
                EnumValueDefinition {
                    description: None,
                    value: Positioned::new(Name::new("ASC"), dummy_position),
                    directives: vec![],
                },
                dummy_position,
            ),
            Positioned::new(
                EnumValueDefinition {
                    description: None,
                    value: Positioned::new(Name::new("DESC"), dummy_position),
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
            name: Positioned::new(Name::new("SortOrder"), dummy_position),
            directives: vec![Positioned::new(
                ConstDirective {
                    name: Positioned::new(Name::new("internal"), dummy_position),
                    arguments: vec![],
                },
                dummy_position,
            )],
            kind: TypeKind::Enum(sort_enum_type),
        },
        dummy_position,
    ))
}

fn create_sort_order_input_obj(
    obj: &ObjectType,
    entity_name: &Name,
) -> Option<TypeSystemDefinition> {
    let dummy_position = Pos {
        line: usize::MAX,
        column: usize::MAX,
    };
    let sortable_fields = obj
        .fields
        .iter()
        .filter(|f| {
            if let BaseType::Named(base_type) = &f.node.ty.node.base {
                SORTABLE_SCALAR_TYPES.contains(base_type.as_str())
            } else {
                false
            }
        })
        .collect::<Vec<&Positioned<FieldDefinition>>>();

    if sortable_fields.is_empty() {
        return None;
    }

    let input_val_defs = sortable_fields
        .into_iter()
        .map(|f| {
            Positioned::new(
                InputValueDefinition {
                    description: None,
                    name: f.node.name.clone(),
                    ty: Positioned::new(
                        Type {
                            base: BaseType::Named(Name::new("SortOrder")),
                            nullable: true,
                        },
                        dummy_position,
                    ),
                    default_value: None,
                    directives: vec![Positioned::new(
                        ConstDirective {
                            name: Positioned::new(Name::new("internal"), dummy_position),
                            arguments: vec![],
                        },
                        dummy_position,
                    )],
                },
                dummy_position,
            )
        })
        .collect::<Vec<Positioned<InputValueDefinition>>>();

    let sort_input_obj = InputObjectType {
        fields: input_val_defs,
    };

    Some(TypeSystemDefinition::Type(Positioned::new(
        TypeDefinition {
            extend: false,
            description: None,
            name: Positioned::new(
                Name::new(format!("{}OrderInput", entity_name)),
                dummy_position,
            ),
            directives: vec![Positioned::new(
                ConstDirective {
                    name: Positioned::new(Name::new("internal"), dummy_position),
                    arguments: vec![],
                },
                dummy_position,
            )],
            kind: TypeKind::InputObject(sort_input_obj),
        },
        dummy_position,
    )))
}

fn create_comparison_obj_for_filtering() -> TypeSystemDefinition {
    let dummy_position = Pos {
        line: usize::MAX,
        column: usize::MAX,
    };

    let comparison_obj = InputObjectType {
        fields: vec![
            Positioned::new(
                InputValueDefinition {
                    description: None,
                    name: Positioned::new(Name::new("equals"), dummy_position),
                    ty: Positioned::new(
                        Type {
                            base: BaseType::Named(Name::new("String")),
                            nullable: true,
                        },
                        dummy_position,
                    ),
                    default_value: None,
                    directives: vec![Positioned::new(
                        ConstDirective {
                            name: Positioned::new(Name::new("internal"), dummy_position),
                            arguments: vec![],
                        },
                        dummy_position,
                    )],
                },
                dummy_position,
            ),
            Positioned::new(
                InputValueDefinition {
                    description: None,
                    name: Positioned::new(Name::new("gt"), dummy_position),
                    ty: Positioned::new(
                        Type {
                            base: BaseType::Named(Name::new("String")),
                            nullable: true,
                        },
                        dummy_position,
                    ),
                    default_value: None,
                    directives: vec![Positioned::new(
                        ConstDirective {
                            name: Positioned::new(Name::new("internal"), dummy_position),
                            arguments: vec![],
                        },
                        dummy_position,
                    )],
                },
                dummy_position,
            ),
            Positioned::new(
                InputValueDefinition {
                    description: None,
                    name: Positioned::new(Name::new("gte"), dummy_position),
                    ty: Positioned::new(
                        Type {
                            base: BaseType::Named(Name::new("String")),
                            nullable: true,
                        },
                        dummy_position,
                    ),
                    default_value: None,
                    directives: vec![Positioned::new(
                        ConstDirective {
                            name: Positioned::new(Name::new("internal"), dummy_position),
                            arguments: vec![],
                        },
                        dummy_position,
                    )],
                },
                dummy_position,
            ),
            Positioned::new(
                InputValueDefinition {
                    description: None,
                    name: Positioned::new(Name::new("lt"), dummy_position),
                    ty: Positioned::new(
                        Type {
                            base: BaseType::Named(Name::new("String")),
                            nullable: true,
                        },
                        dummy_position,
                    ),
                    default_value: None,
                    directives: vec![Positioned::new(
                        ConstDirective {
                            name: Positioned::new(Name::new("internal"), dummy_position),
                            arguments: vec![],
                        },
                        dummy_position,
                    )],
                },
                dummy_position,
            ),
            Positioned::new(
                InputValueDefinition {
                    description: None,
                    name: Positioned::new(Name::new("lte"), dummy_position),
                    ty: Positioned::new(
                        Type {
                            base: BaseType::Named(Name::new("String")),
                            nullable: true,
                        },
                        dummy_position,
                    ),
                    default_value: None,
                    directives: vec![Positioned::new(
                        ConstDirective {
                            name: Positioned::new(Name::new("internal"), dummy_position),
                            arguments: vec![],
                        },
                        dummy_position,
                    )],
                },
                dummy_position,
            ),
        ],
    };

    TypeSystemDefinition::Type(Positioned::new(
        TypeDefinition {
            extend: false,
            description: None,
            name: Positioned::new(Name::new("ComparisonInput"), dummy_position),
            directives: vec![Positioned::new(
                ConstDirective {
                    name: Positioned::new(Name::new("internal"), dummy_position),
                    arguments: vec![],
                },
                dummy_position,
            )],
            kind: TypeKind::InputObject(comparison_obj),
        },
        dummy_position,
    ))
}

fn create_filter_input_obj_for_entity(
    obj: &ObjectType,
    entity_name: &Name,
) -> Option<TypeSystemDefinition> {
    let dummy_position = Pos {
        line: usize::MAX,
        column: usize::MAX,
    };

    let filterable_fields = obj
        .fields
        .iter()
        .filter(|f| check_for_directive(&f.node.directives, "filterable"))
        .collect::<Vec<&Positioned<FieldDefinition>>>();

    if filterable_fields.is_empty() {
        return None;
    }

    let mut input_val_defs = filterable_fields
        .into_iter()
        .map(|f| {
            Positioned::new(
                InputValueDefinition {
                    description: None,
                    name: f.node.name.clone(),
                    ty: Positioned::new(
                        Type {
                            base: BaseType::Named(Name::new("ComparisonInput")),
                            nullable: true,
                        },
                        dummy_position,
                    ),
                    default_value: None,
                    directives: vec![Positioned::new(
                        ConstDirective {
                            name: Positioned::new(Name::new("internal"), dummy_position),
                            arguments: vec![],
                        },
                        dummy_position,
                    )],
                },
                dummy_position,
            )
        })
        .collect::<Vec<Positioned<InputValueDefinition>>>();

    // Allow for combinations of filters
    input_val_defs.push(Positioned::new(
        InputValueDefinition {
            description: None,
            name: Positioned::new(Name::new("and"), dummy_position),
            ty: Positioned::new(
                Type {
                    base: BaseType::Named(Name::new(format!(
                        "{}FilterInput",
                        entity_name
                    ))),
                    nullable: true,
                },
                dummy_position,
            ),
            default_value: None,
            directives: vec![Positioned::new(
                ConstDirective {
                    name: Positioned::new(Name::new("internal"), dummy_position),
                    arguments: vec![],
                },
                dummy_position,
            )],
        },
        dummy_position,
    ));

    input_val_defs.push(Positioned::new(
        InputValueDefinition {
            description: None,
            name: Positioned::new(Name::new("or"), dummy_position),
            ty: Positioned::new(
                Type {
                    base: BaseType::Named(Name::new(format!(
                        "{}FilterInput",
                        entity_name
                    ))),
                    nullable: true,
                },
                dummy_position,
            ),
            default_value: None,
            directives: vec![Positioned::new(
                ConstDirective {
                    name: Positioned::new(Name::new("internal"), dummy_position),
                    arguments: vec![],
                },
                dummy_position,
            )],
        },
        dummy_position,
    ));

    input_val_defs.push(Positioned::new(
        InputValueDefinition {
            description: None,
            name: Positioned::new(Name::new("not"), dummy_position),
            ty: Positioned::new(
                Type {
                    base: BaseType::Named(Name::new(format!(
                        "{}FilterInput",
                        entity_name
                    ))),
                    nullable: true,
                },
                dummy_position,
            ),
            default_value: None,
            directives: vec![Positioned::new(
                ConstDirective {
                    name: Positioned::new(Name::new("internal"), dummy_position),
                    arguments: vec![],
                },
                dummy_position,
            )],
        },
        dummy_position,
    ));

    let filter_input_obj = InputObjectType {
        fields: input_val_defs,
    };

    Some(TypeSystemDefinition::Type(Positioned::new(
        TypeDefinition {
            extend: false,
            description: None,
            name: Positioned::new(
                Name::new(format!("{}FilterInput", entity_name)),
                dummy_position,
            ),
            directives: vec![Positioned::new(
                ConstDirective {
                    name: Positioned::new(Name::new("internal"), dummy_position),
                    arguments: vec![],
                },
                dummy_position,
            )],
            kind: TypeKind::InputObject(filter_input_obj),
        },
        dummy_position,
    )))
}

fn create_edge_type(entity_def: &Positioned<TypeDefinition>) -> TypeSystemDefinition {
    let edge_obj_type = ObjectType {
        implements: vec![],
        fields: vec![
            Positioned::position_node(
                entity_def,
                FieldDefinition {
                    description: None,
                    name: Positioned::position_node(entity_def, Name::new("node")),
                    arguments: vec![],
                    ty: Positioned::position_node(
                        entity_def,
                        Type {
                            base: BaseType::Named(entity_def.node.name.node.clone()),
                            nullable: false,
                        },
                    ),
                    directives: vec![],
                },
            ),
            Positioned::position_node(
                entity_def,
                FieldDefinition {
                    description: None,
                    name: Positioned::position_node(entity_def, Name::new("cursor")),
                    arguments: vec![],
                    ty: Positioned::position_node(
                        entity_def,
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
        entity_def,
        TypeDefinition {
            extend: false,
            description: None,
            name: Positioned::position_node(
                entity_def,
                Name::new(format!("{}Edge", entity_def.node.name.node.clone())),
            ),
            directives: vec![
                Positioned::position_node(
                    entity_def,
                    ConstDirective {
                        name: Positioned::position_node(
                            entity_def,
                            Name::new("internal"),
                        ),
                        arguments: vec![],
                    },
                ),
                Positioned::position_node(
                    entity_def,
                    ConstDirective {
                        name: Positioned::position_node(entity_def, Name::new("edge")),
                        arguments: vec![],
                    },
                ),
            ],
            kind: TypeKind::Object(edge_obj_type),
        },
    ))
}

/// Generate connection type defintion for an entity.
fn create_connection_type_def(
    name: &Name,
    base_entity_queryable_at_root: bool,
) -> TypeSystemDefinition {
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

    let mut directives = vec![
        Positioned::new(
            ConstDirective {
                name: Positioned::new(Name::new("internal"), dummy_position),
                arguments: vec![],
            },
            dummy_position,
        ),
        Positioned::new(
            ConstDirective {
                name: Positioned::new(Name::new("connection"), dummy_position),
                arguments: vec![],
            },
            dummy_position,
        ),
    ];

    if base_entity_queryable_at_root {
        directives.push(Positioned::new(
            ConstDirective {
                name: Positioned::new(Name::new("queryable"), dummy_position),
                arguments: vec![],
            },
            dummy_position,
        ))
    }

    TypeSystemDefinition::Type(Positioned::new(
        TypeDefinition {
            extend: false,
            description: None,
            name: Positioned::new(
                Name::new(format!("{}Connection", name.clone())),
                dummy_position,
            ),
            directives,
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

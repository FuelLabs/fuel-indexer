use std::collections::HashSet;

use async_graphql::dynamic::{
    Enum, EnumItem, Field, FieldFuture, FieldValue, InputObject, InputValue, Object,
    ResolverContext, Scalar, Schema as DynamicSchema,
    SchemaBuilder as DynamicSchemaBuilder, TypeRef, Union,
};
use async_graphql_parser::types::BaseType;

use fuel_indexer_schema::db::tables::IndexerSchema;
use lazy_static::lazy_static;

use crate::GraphqlResult;

lazy_static! {
    /// Scalar types supported by the Fuel indexer. These should always stay up-to-date
    /// with fuel-indexer-schema/src/base.graphql.
    static ref SCALAR_TYPES: HashSet<&'static str> = HashSet::from([
        "Address",
        "AssetId",
        "Boolean",
        "Bytes",
        "Bytes32",
        "Bytes4",
        "Bytes64",
        "Bytes8",
        "ContractId",
        "I128",
        "I16",
        "I32",
        "I64",
        "I8",
        "ID",
        "Identity",
        "Json",
        "U128",
        "U16",
        "U32",
        "U64",
        "U8",
        "UID",
    ]);

    /// Entity types that should be ignored when building the dynamic schema,
    /// so that they do not appear in the generated documentation. This is done
    /// to hide internal Fuel indexer entity types.
    static ref IGNORED_ENTITY_TYPES: HashSet<&'static str> =
        HashSet::from(["IndexMetadataEntity"]);

    /// Entity fields that should be ignored when building the dynamic schema,
    /// so that they do not appear in the generated documentation. This is done
    /// to hide internal Fuel indexer entity fields.
    static ref IGNORED_ENTITY_FIELD_TYPES: HashSet<&'static str> =
        HashSet::from(["object"]);
}

/// Build a dynamic schema. This allows for introspection, which allows for extensive
/// auto-documentation and code suggestions.
pub fn build_dynamic_schema(schema: &IndexerSchema) -> GraphqlResult<DynamicSchema> {
    // Register scalars into dynamic schema so that users are aware of their existence.
    let mut schema_builder: DynamicSchemaBuilder = SCALAR_TYPES.iter().fold(
        DynamicSchema::build("Query", None, None).introspection_only(),
        |sb, scalar| {
            // These types come pre-included in SchemaBuilder.
            if *scalar == "Boolean" || *scalar == "ID" {
                sb
            } else {
                sb.register(Scalar::new(*scalar))
            }
        },
    );

    // async-graphql requires a root query object so that the base entity
    // fields can be queried against. This QueryRoot does not appear anywhere
    // in the generated documentation nor is it required for the user to create.
    let mut query_root = Object::new("Query");
    for (query_key, field_def) in schema.parsed().queries() {
        let base_type = &field_def.ty.node.base;
        let nullable = field_def.ty.node.nullable;

        let field_type = match base_type {
            BaseType::Named(named_type) => {
                if nullable {
                    TypeRef::named(named_type.to_string())
                } else {
                    TypeRef::named_nn(named_type.to_string())
                }
            }
            BaseType::List(list_type) => {
                let inner_base_type = list_type.base.to_string();
                let nullable_inner = list_type.nullable;

                if nullable && nullable_inner {
                    TypeRef::named_list(inner_base_type)
                } else if nullable && !nullable_inner {
                    TypeRef::named_nn_list(inner_base_type)
                } else if !nullable && nullable_inner {
                    TypeRef::named_list_nn(inner_base_type)
                } else {
                    TypeRef::named_nn_list_nn(inner_base_type)
                }
            }
        };

        let mut field =
            Field::new(query_key, field_type, move |_ctx: ResolverContext| {
                return FieldFuture::new(async move { Ok(Some(FieldValue::value(1))) });
            });

        for arg in field_def.arguments.iter() {
            let name = &arg.node.name.node;
            let base_type = &arg.node.ty.node.base;
            let nullable = arg.node.ty.node.nullable;

            let arg_type = match base_type {
                BaseType::Named(named_type) => {
                    if nullable {
                        TypeRef::named(named_type.to_string())
                    } else {
                        TypeRef::named_nn(named_type.to_string())
                    }
                }
                BaseType::List(list_type) => {
                    let inner_base_type = list_type.base.to_string();
                    let nullable_inner = list_type.nullable;

                    if nullable && nullable_inner {
                        TypeRef::named_list(inner_base_type)
                    } else if nullable && !nullable_inner {
                        TypeRef::named_nn_list(inner_base_type)
                    } else if !nullable && nullable_inner {
                        TypeRef::named_list_nn(inner_base_type)
                    } else {
                        TypeRef::named_nn_list_nn(inner_base_type)
                    }
                }
            };

            let input_value = InputValue::new(name.to_string(), arg_type);
            field = field.argument(input_value);
        }

        query_root = query_root.field(field);
    }

    for (entity_name, obj_type) in schema.parsed().objects() {
        if IGNORED_ENTITY_TYPES.contains(&entity_name.as_str()) {
            continue;
        }

        let mut fields = Vec::new();
        for field_def in obj_type.fields.iter() {
            let field_name = field_def.node.name.to_string();
            if IGNORED_ENTITY_FIELD_TYPES.contains(&field_name.as_str()) {
                continue;
            }

            let base_field_type = &field_def.node.ty.node.base;
            let nullable = field_def.node.ty.node.nullable;

            let field_type = match base_field_type {
                BaseType::Named(type_name) => {
                    if nullable {
                        // TODO: If we do not check for virtual types,
                        // enums become recursively self-referential and the playground
                        // will report errors related to enum subfields not being
                        // supplied.
                        //
                        //
                        // For now, setting them to a String type does not
                        // cause errors, but we should decide what the final process is.
                        if schema.parsed().is_virtual_typedef(type_name) {
                            TypeRef::named(TypeRef::STRING)
                        } else {
                            TypeRef::named(type_name.to_string())
                        }
                    } else if schema.parsed().is_virtual_typedef(type_name) {
                        TypeRef::named_nn(TypeRef::STRING)
                    } else {
                        TypeRef::named_nn(type_name.to_string())
                    }
                }
                BaseType::List(list_type) => {
                    let inner_base_type = list_type.base.to_string();
                    let nullable_inner = list_type.nullable;

                    if nullable && nullable_inner {
                        TypeRef::named_list(inner_base_type)
                    } else if nullable && !nullable_inner {
                        TypeRef::named_nn_list(inner_base_type)
                    } else if !nullable && nullable_inner {
                        TypeRef::named_list_nn(inner_base_type)
                    } else {
                        TypeRef::named_nn_list_nn(inner_base_type)
                    }
                }
            };

            let field = Field::new(
                field_name.to_string(),
                field_type,
                move |_ctx: ResolverContext| {
                    return FieldFuture::new(
                        async move { Ok(Some(FieldValue::value(1))) },
                    );
                },
            );

            fields.push(field);
        }

        // Create object using all of the fields that were constructed for the entity
        // and repeat the same process in order to allow for introspection-related
        // functionality at the root query level.
        let obj = fields
            .into_iter()
            .fold(Object::new(entity_name.clone()), |obj, f| obj.field(f));

        schema_builder = schema_builder.register(obj);
    }

    for (input_obj_name, input_obj) in schema.parsed().input_objs() {
        let mut fields: Vec<InputValue> = vec![];
        for field_def in input_obj.fields.iter() {
            let field_name = field_def.node.name.to_string();
            if IGNORED_ENTITY_FIELD_TYPES.contains(&field_name.as_str()) {
                continue;
            }

            let base_field_type = &field_def.node.ty.node.base;
            let nullable = field_def.node.ty.node.nullable;

            let field_type = match base_field_type {
                BaseType::Named(type_name) => {
                    if nullable {
                        TypeRef::named(type_name.to_string())
                    } else {
                        TypeRef::named_nn(type_name.to_string())
                    }
                }
                BaseType::List(list_type) => {
                    let inner_base_type = list_type.base.to_string();
                    let nullable_inner = list_type.nullable;

                    if nullable && nullable_inner {
                        TypeRef::named_list(inner_base_type)
                    } else if nullable && !nullable_inner {
                        TypeRef::named_nn_list(inner_base_type)
                    } else if !nullable && nullable_inner {
                        TypeRef::named_list_nn(inner_base_type)
                    } else {
                        TypeRef::named_nn_list_nn(inner_base_type)
                    }
                }
            };

            let field = InputValue::new(field_name.to_string(), field_type);

            fields.push(field);
        }

        let io = fields
            .into_iter()
            .fold(InputObject::new(input_obj_name), |io, input_val| {
                io.field(input_val)
            });
        schema_builder = schema_builder.register(io);
    }

    for (enum_name, member_map) in schema.parsed().enum_member_map() {
        let e = member_map.iter().fold(Enum::new(enum_name), |e, e_val| {
            e.item(EnumItem::new(e_val.as_str()))
        });
        schema_builder = schema_builder.register(e);
    }

    for (union_name, member_map) in schema.parsed().union_member_map() {
        let u = member_map.iter().fold(Union::new(union_name), |u, u_val| {
            u.possible_type(u_val.as_str())
        });
        schema_builder = schema_builder.register(u);
    }

    schema_builder = schema_builder.register(query_root);

    Ok(schema_builder.finish()?)
}

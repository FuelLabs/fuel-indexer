use std::sync::Arc;

use async_graphql::dynamic::{
    Field, FieldFuture, FieldValue, Object, ResolverContext, Scalar,
    Schema as DynamicSchema, SchemaBuilder as DynamicSchemaBuilder, TypeRef,
};
use async_graphql_parser::{
    parse_schema,
    types::{BaseType, TypeKind, TypeSystemDefinition},
};
use fuel_indexer_database::{queries, IndexerConnectionPool};
use fuel_indexer_schema::db::tables::Schema;

use crate::graphql::{GraphqlError, GraphqlQueryBuilder};

pub async fn build_dynamic_schema(
    schema: Schema,
    pool: IndexerConnectionPool,
) -> Result<DynamicSchema, GraphqlError> {
    let indexer_schema = parse_schema(schema.schema.clone()).unwrap();

    let mut query_root = Object::new("QueryRoot");
    let mut objects = Vec::new();

    for def in indexer_schema.definitions.iter() {
        match def {
            TypeSystemDefinition::Schema(_) => {}
            TypeSystemDefinition::Type(type_def_wrapper) => {
                let type_def = &type_def_wrapper.node;

                match &type_def.kind {
                    TypeKind::Scalar => {}
                    TypeKind::Object(obj_type) => {
                        let (is_query_root, mut obj): (bool, Option<Object>) =
                            match type_def.name.node.as_str() {
                                "QueryRoot" => (true, None),
                                name => (false, Some(Object::new(name))),
                            };

                        for field in obj_type.fields.iter() {
                            let field_name = field.node.name.to_string();
                            let base_field_type = &field.node.ty.node.base;
                            let nullable = field.node.ty.node.nullable;

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

                            let schema = Arc::new(schema.clone());

                            let field = if is_query_root {
                                Field::new(
                                    field_name,
                                    field_type,
                                    move |ctx: ResolverContext| {
                                        let schema = schema.clone();
                                        FieldFuture::new(async move {
                                            let pool =
                                                ctx.data::<IndexerConnectionPool>()?;

                                            let query = GraphqlQueryBuilder::new(
                                                &schema,
                                                ctx.data::<String>()?,
                                            )?
                                            .build()?;

                                            let queries = query
                                                .as_sql(&schema, pool.database_type())?
                                                .join(";\n");

                                            let mut conn = pool.acquire().await?;

                                            let result =
                                                queries::run_query(&mut conn, queries)
                                                    .await?;

                                            // TODO: Remove unwraps
                                            let list =
                                                result.as_array().unwrap().to_owned();
                                            let first_result =
                                                list.get(0).unwrap().to_owned();
                                            Ok(Some(FieldValue::owned_any(first_result)))
                                        })
                                    },
                                )
                            } else {
                                Field::new(
                                    field_name,
                                    field_type,
                                    move |_ctx: ResolverContext| {
                                        FieldFuture::new(
                                            async move { Ok(FieldValue::none()) },
                                        )
                                    },
                                )
                            };

                            if is_query_root {
                                query_root = query_root.field(field);
                            } else if let Some(o) = obj {
                                obj = Some(o.field(field));
                            }
                        }

                        if !is_query_root {
                            if let Some(o) = obj {
                                objects.push(o);
                            }
                        }
                    }
                    TypeKind::Interface(_) => {}
                    TypeKind::Union(_) => {}
                    TypeKind::Enum(_) => {}
                    TypeKind::InputObject(_) => {}
                }
            }
            TypeSystemDefinition::Directive(_) => {}
        }
    }

    // TODO: Add scalar here for large number types
    let mut schema_builder: DynamicSchemaBuilder =
        DynamicSchema::build(query_root.type_name(), None, None)
            .register(Scalar::new("Address"))
            .register(Scalar::new("AssetId"))
            .register(Scalar::new("Bytes4"))
            .register(Scalar::new("Bytes8"))
            .register(Scalar::new("Bytes32"))
            .register(Scalar::new("Bytes64"))
            .register(Scalar::new("Int4"))
            .register(Scalar::new("Int8"))
            .register(Scalar::new("Int16"))
            .register(Scalar::new("UInt4"))
            .register(Scalar::new("UInt8"))
            .register(Scalar::new("UInt16"))
            .register(Scalar::new("Timestamp"))
            .register(Scalar::new("ContractId"))
            .register(Scalar::new("Salt"))
            .register(Scalar::new("Json"))
            .register(Scalar::new("MessageId"))
            .register(Scalar::new("Charfield"))
            .register(Scalar::new("Identity"))
            .register(Scalar::new("Blob"))
            .register(query_root)
            .data(pool);

    for obj in objects {
        schema_builder = schema_builder.register(obj);
    }

    match schema_builder.finish() {
        Ok(schema) => Ok(schema),
        Err(e) => Err(GraphqlError::DynamicSchemaBuildError(e)),
    }
}

use std::collections::{HashMap, HashSet};

use async_graphql::{
    dynamic::{
        Enum, Field, FieldFuture, FieldValue, InputObject, InputValue, Object,
        ResolverContext, Scalar, Schema as DynamicSchema,
        SchemaBuilder as DynamicSchemaBuilder, SchemaError, TypeRef,
    },
    Request,
};
use async_graphql_parser::types::{BaseType, Type};
use async_graphql_value::Name;
use fuel_indexer_database::{queries, IndexerConnectionPool};
use fuel_indexer_schema::db::tables::Schema;
use lazy_static::lazy_static;
use serde_json::Value;

use crate::graphql::{GraphqlError, GraphqlQueryBuilder, GraphqlResult};

lazy_static! {
    /// Scalar types supported by the Fuel indexer. These should always stay up-to-date
    /// with fuel-indexer-schema/src/base.graphql.
    static ref SCALAR_TYPES: HashSet<&'static str> = HashSet::from([
        "Address",
        "AssetId",
        "Blob",
        "BlockHeight",
        "Boolean",
        "Bytes32",
        "Bytes4",
        "Bytes64",
        "Bytes8",
        "Charfield",
        "Color",
        "ContractId",
        "HexString",
        "ID",
        "Identity",
        "Int1",
        "Int16",
        "Int4",
        "Int8",
        "Json",
        "MessageId",
        "Nonce",
        "Salt",
        "Signature",
        "Tai64Timestamp",
        "Timestamp",
        "TxId",
        "UInt1",
        "UInt16",
        "UInt4",
        "UInt8",
    ]);

    /// Scalar types that are represented by a numeric type. This ensures that the
    /// value type provided for a field filter matches the type of the scalar itself.
    static ref NUMERIC_SCALAR_TYPES: HashSet<&'static str> = HashSet::from([
        "Int16",
        "Int4",
        "Int8",
        "Timestamp",
        "Tai64Timestamp",
        "UInt16",
        "UInt4",
        "UInt8",
    ]);

    /// Scalar types that are represented by a string type. This ensures that the
    /// value type provided for a field filter matches the type of the scalar itself.
    static ref STRING_SCALAR_TYPES: HashSet<&'static str> = HashSet::from([
        "Address",
        "AssetId",
        "Blob",
        "Bytes32",
        "Bytes4",
        "Bytes64",
        "Bytes64",
        "Bytes8",
        "Charfield",
        "Color",
        "ContractId",
        "HexString",
        "ID",
        "Identity",
        "Json",
        "MessageId",
        "Nonce",
        "Salt",
        "Signature",
    ]);

    /// Scalar types that can be sorted.
    static ref SORTABLE_SCALAR_TYPES: HashSet<&'static str> = HashSet::from([
        "Address",
        "AssetId",
        "Charfield",
        "Color",
        "ContractId",
        "HexString",
        "ID",
        "Identity",
        "Int16",
        "Int4",
        "Int8",
        "MessageId",
        "Nonce",
        "Salt",
        "Signature",
        "Tai64Timestamp",
        "Timestamp",
        "UInt16",
        "UInt4",
        "UInt8",
    ]);

    /// Entity types that should be ignored when building the dynamic schema,
    /// so that they do not appear in the generated documentation. This is done
    /// to hide internal Fuel indexer entity types.
    static ref IGNORED_ENTITY_TYPES: HashSet<&'static str> =
        HashSet::from(["IndexMetadataEntity", "QueryRoot"]);

    /// Entity fields that should be ignored when building the dynamic schema,
    /// so that they do not appear in the generated documentation. This is done
    /// to hide internal Fuel indexer entity fields.
    static ref IGNORED_ENTITY_FIELD_TYPES: HashSet<&'static str> =
        HashSet::from(["object"]);
}

/// Execute user query and return results.
pub async fn execute_query(
    dynamic_request: Request,
    dynamic_schema: DynamicSchema,
    user_query: String,
    pool: IndexerConnectionPool,
    schema: Schema,
) -> GraphqlResult<Value> {
    // Because the schema types from async-graphql expect each field to be resolved
    // separately, it became untenable to use the .execute() method of the dynamic
    // schema itself to resolve queries. Instead, we set it to only resolve
    // introspection queries and then pass any non-introspection queries to our
    // custom query resolver.
    match dynamic_request.operation_name.as_deref() {
        Some("IntrospectionQuery") | Some("introspectionquery") => {
            let introspection_results = dynamic_schema.execute(dynamic_request).await;
            let data = introspection_results.data.into_json()?;

            Ok(data)
        }
        Some(_) | None => {
            let query =
                GraphqlQueryBuilder::new(&schema, user_query.as_str())?.build()?;

            let queries = query.as_sql(&schema, pool.database_type())?.join(";\n");
            let mut conn = match pool.acquire().await {
                Ok(c) => c,
                Err(e) => return Err(GraphqlError::QueryError(e.to_string())),
            };

            match queries::run_query(&mut conn, queries).await {
                Ok(r) => Ok(r),
                Err(e) => Err(GraphqlError::QueryError(e.to_string())),
            }
        }
    }
}

/// Build a dynamic schema. This allows for introspection, which allows for extensive
/// auto-documentation and code suggestions.
pub fn build_dynamic_schema(schema: &Schema) -> GraphqlResult<DynamicSchema> {
    // Register scalars into dynamic schema so that users are aware of their existence.
    let mut schema_builder: DynamicSchemaBuilder = SCALAR_TYPES.iter().fold(
        DynamicSchema::build("QueryRoot", None, None).introspection_only(),
        |sb, scalar| {
            // These types come pre-included in SchemaBuilder.
            if *scalar == "Boolean" || *scalar == "ID" {
                sb
            } else {
                sb.register(Scalar::new(*scalar))
            }
        },
    );

    let mut input_objects = Vec::new();
    let mut filter_object_list = Vec::new();
    let mut filter_tracker = HashMap::new();
    let mut sort_object_list = Vec::new();
    let mut sorter_tracker = HashMap::new();

    // async-graphql requires a root query object so that the base entity
    // fields can be queried against. This QueryRoot does not appear anywhere
    // in the generated documentation nor is it required for the user to create.
    let mut query_root = Object::new("QueryRoot");

    let sort_enum = Enum::new("SortOrder").item("asc").item("desc");

    for (entity_type, field_map) in &schema.fields {
        if IGNORED_ENTITY_TYPES.contains(&entity_type.as_str()) {
            continue;
        }

        // Input values are stored for each field so that we can create
        // a final input object for filtering and sorting a field.
        let mut filter_input_vals = Vec::new();
        let mut sort_input_vals = Vec::new();

        // Field names will be added to this enum in order to allow
        // for filtering on the column itself, i.e. "has" operator.
        let mut object_field_enum = Enum::new(format!("{entity_type}Fields"));

        for (field_name, field_type) in field_map.clone() {
            if IGNORED_ENTITY_FIELD_TYPES.contains(&field_name.as_str()) {
                continue;
            }

            let (field_filter_input_val, mut field_input_objects, sort_input_val) =
                create_input_values_and_objects_for_field(
                    field_name.clone(),
                    field_type,
                    entity_type.clone(),
                    &sort_enum,
                )?;

            filter_input_vals.push(field_filter_input_val);
            input_objects.append(&mut field_input_objects);

            if let Some(input_val) = sort_input_val {
                sort_input_vals.push(input_val);
            }

            object_field_enum = object_field_enum.item(field_name);
        }

        // Fold corresponding input values into objects for that particular field.
        let filter_object = filter_input_vals
            .into_iter()
            .fold(
                InputObject::new(format!("{entity_type}Filter")),
                |input_obj, input_val| input_obj.field(input_val),
            )
            .field(InputValue::new(
                "has",
                TypeRef::named_nn_list(object_field_enum.type_name()),
            ));

        let sort_object = sort_input_vals.into_iter().fold(
            InputObject::new(format!("{entity_type}Sort")),
            |input_obj, input_val| input_obj.field(input_val),
        );

        // For some reason, async-graphql does not implement the Hash trait on any of the
        // type that we need for dynamic schemas. So we are essentially making a hash table
        // ourselves for the filter and sort objects.
        filter_object_list.push(filter_object);
        filter_tracker.insert(entity_type.to_string(), filter_object_list.len() - 1);

        sort_object_list.push(sort_object);
        sorter_tracker.insert(entity_type.to_string(), sort_object_list.len() - 1);

        // Additionally, because we cannot refer to the object fields directly and
        // associate the field arguments to them, we iterate through the fields a
        // second time and construct the fields for the dynamic schema and add the
        // field arguments as well.
        let mut fields = Vec::new();
        for (field_name, field_type) in field_map {
            if IGNORED_ENTITY_FIELD_TYPES.contains(&field_name.as_str()) {
                continue;
            }

            if let Some(field_def) = Type::new(field_type) {
                let base_field_type = &field_def.base;
                let nullable = field_def.nullable;

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

                let field = create_field_with_assoc_args(
                    field_name.to_string(),
                    field_type,
                    base_field_type,
                    &filter_tracker,
                    &filter_object_list,
                    &sorter_tracker,
                    &sort_object_list,
                );

                fields.push(field);
            }
        }

        // Create object using all of the fields that were constructed for the entity
        // and repeat the same process in order to allow for introspection-related
        // functionality at the root query level.
        let obj = fields
            .into_iter()
            .fold(Object::new(entity_type.clone()), |obj, f| obj.field(f));

        // Create field for entity object and add it to root level query object.
        let field = create_field_with_assoc_args(
            entity_type.to_string().to_lowercase(),
            TypeRef::named(obj.type_name()),
            &BaseType::Named(Name::new(obj.type_name())),
            &filter_tracker,
            &filter_object_list,
            &sorter_tracker,
            &sort_object_list,
        );
        if !SCALAR_TYPES.contains(&obj.type_name()) {
            query_root = query_root.field(field);
        }

        schema_builder = schema_builder.register(obj).register(object_field_enum);
    }

    // In order for the schema to successfully use the input objects
    // that make up the filter and sort arguments, the objects have to
    // be registered with the schema.
    for filter_obj in filter_object_list {
        schema_builder = schema_builder.register(filter_obj);
    }

    for sort_obj in sort_object_list {
        schema_builder = schema_builder.register(sort_obj);
    }

    for io in input_objects {
        schema_builder = schema_builder.register(io);
    }

    schema_builder = schema_builder.register(sort_enum);
    schema_builder = schema_builder.register(query_root);

    Ok(schema_builder.finish()?)
}

/// Create input values and objects that are used to build introspection information for a field.
fn create_input_values_and_objects_for_field(
    field_name: String,
    field_type: String,
    entity_type: String,
    sort_enum: &Enum,
) -> GraphqlResult<(InputValue, Vec<InputObject>, Option<InputValue>)> {
    let field_type =
        Type::new(&field_type).ok_or(GraphqlError::DynamicSchemaBuildError(
            SchemaError::from("Could not create type defintion from field type string"),
        ))?;

    match field_type.base {
        BaseType::Named(field_type) => {
            let (field_filter_input_val, field_input_objects) =
                create_filter_val_and_objects_for_field(
                    &field_name,
                    field_type.as_str(),
                    entity_type.as_str(),
                );

            if SORTABLE_SCALAR_TYPES.contains(field_type.as_str()) {
                let sort_input_val =
                    InputValue::new(field_name, TypeRef::named(sort_enum.type_name()));
                return Ok((
                    field_filter_input_val,
                    field_input_objects,
                    Some(sort_input_val),
                ));
            }

            Ok((field_filter_input_val, field_input_objects, None))
        }
        // TODO: Do the same as above, but with list type
        BaseType::List(_) => unimplemented!("List types are not currently supported"),
    }
}

fn create_field_with_assoc_args(
    field_name: String,
    field_type_ref: TypeRef,
    base_field_type: &BaseType,
    filter_tracker: &HashMap<String, usize>,
    filter_object_list: &[InputObject],
    sorter_tracker: &HashMap<String, usize>,
    sort_object_list: &[InputObject],
) -> Field {
    // Because the dynamic schema is set to only resolve introspection
    // queries, we set the resolvers to return a dummy value.
    let mut field =
        Field::new(field_name, field_type_ref, move |_ctx: ResolverContext| {
            return FieldFuture::new(async move { Ok(Some(FieldValue::value(1))) });
        });

    match base_field_type {
        BaseType::Named(field_type) => {
            if !SCALAR_TYPES.contains(field_type.as_str()) {
                if let Some(idx) = filter_tracker.get(&field_type.to_string()) {
                    let object_filter_arg = InputValue::new(
                        "filter",
                        TypeRef::named(filter_object_list[*idx].type_name()),
                    );
                    field = field.argument(object_filter_arg)
                }

                if let Some(idx) = sorter_tracker.get(&field_type.to_string()) {
                    let object_sort_arg = InputValue::new(
                        "order",
                        TypeRef::named(sort_object_list[*idx].type_name()),
                    );
                    field = field.argument(object_sort_arg);
                }

                let offset_arg = InputValue::new("offset", TypeRef::named(TypeRef::INT));

                let limit_arg = InputValue::new("first", TypeRef::named(TypeRef::INT));

                let id_selection_arg =
                    InputValue::new("id", TypeRef::named(TypeRef::STRING));

                field = field
                    .argument(offset_arg)
                    .argument(limit_arg)
                    .argument(id_selection_arg);
            }
        }
        BaseType::List(_) => unimplemented!("List types are not currently supported"),
    }

    field
}

/// Build the filter objects for a particular field. The resultant object
/// will ensure that the correct value type is allowed for the field by
/// passing the input type information in the introspection response.
fn create_filter_val_and_objects_for_field<'a>(
    field_name: &'a str,
    field_type: &'a str,
    obj_name: &'a str,
) -> (InputValue, Vec<InputObject>) {
    let mut input_objs: Vec<InputObject> = Vec::new();

    let filter_arg_type = if NUMERIC_SCALAR_TYPES.contains(field_type) {
        TypeRef::INT
    } else {
        TypeRef::STRING
    };

    // TODO: Add support for logical operators -- https://github.com/FuelLabs/fuel-indexer/issues/917

    let complex_comparison_obj =
        InputObject::new(format!("{obj_name}_{field_name}_ComplexComparisonObject"))
            .field(InputValue::new("min", TypeRef::named_nn(filter_arg_type)))
            .field(InputValue::new("max", TypeRef::named_nn(filter_arg_type)));

    let complete_comparison_obj =
        InputObject::new(format!("{obj_name}{field_name}FilterObject"))
            .field(InputValue::new(
                "between",
                TypeRef::named(complex_comparison_obj.type_name()),
            ))
            .field(InputValue::new("equals", TypeRef::named(filter_arg_type)))
            .field(InputValue::new("gt", TypeRef::named(filter_arg_type)))
            .field(InputValue::new("gte", TypeRef::named(filter_arg_type)))
            .field(InputValue::new("lt", TypeRef::named(filter_arg_type)))
            .field(InputValue::new("lte", TypeRef::named(filter_arg_type)))
            .field(InputValue::new(
                "in",
                TypeRef::named_nn_list(filter_arg_type),
            ));

    let input_val_for_field = InputValue::new(
        field_name,
        TypeRef::named(complete_comparison_obj.type_name()),
    );

    input_objs.append(&mut vec![complex_comparison_obj, complete_comparison_obj]);
    (input_val_for_field, input_objs)
}

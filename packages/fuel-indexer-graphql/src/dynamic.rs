// At first, I thought that I could coerce the async-graphql crate into doing my bidding. It laughed
// at me. Undeterred, I pressed on. "If only it could see my vision," I thought, "it would yield
// to me in an instant." Alas, if I could only return to the naïveté of that time. Oh, to
// be that young developer again; they were so bright...and so foolish. We clashed
// continuously over the period of several moons; with my face illuminated by faint blue
// light, I tried and tried. Thrusts of brilliant engineering were invariably met, at times with
// parries and at times with an added riposte. Bruised but unbeaten, I toiled. I searched for answers
// in the type system of async-graphql. But where I hoped and expected to find solace, I found only
// mockery. "Why wasn't the Hash trait implemented?" I asked aloud. "Why wouldn't someone derive at least
// the Debug trait?" I cursed the gods above and below, and I vowed to mete out my own punishment
// upon the crate (in the form of upstream changes) once I finished...if ever.
//
// An uncountably infinite time passed. Gazing upon the blinking cursor of my editor, I compromised.
// With a heavy sigh and even heavier sense of defeat, I implemented the functionality below. I have
// documented the sites of frustrating battles, of potential improvements, and of innocence lost.

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use async_graphql::{
    dynamic::{
        Enum, Field, FieldFuture, FieldValue, InputObject, InputValue, Object,
        ResolverContext, Scalar, Schema as DynamicSchema,
        SchemaBuilder as DynamicSchemaBuilder, SchemaError, TypeRef,
    },
    futures_util::lock::Mutex,
    Request,
};
use async_graphql_parser::{
    parse_schema,
    types::{BaseType, ObjectType, TypeKind, TypeSystemDefinition},
};
use async_graphql_value::ConstValue;
use fuel_indexer_database::{queries, IndexerConnectionPool};
use fuel_indexer_schema::db::tables::Schema;
use lazy_static::lazy_static;
use serde_json::Value;

use crate::graphql::{GraphqlError, GraphqlQueryBuilder};

lazy_static! {
    static ref SCALAR_TYPES: HashSet<&'static str> = HashSet::from([
        "ID",
        "Address",
        "AssetId",
        "Bytes4",
        "Bytes8",
        "Bytes32",
        "Bytes64",
        "Int4",
        "Int8",
        "Int16",
        "UInt4",
        "UInt8",
        "UInt16",
        "Timestamp",
        "Color",
        "ContractId",
        "Salt",
        "Json",
        "MessageId",
        "Charfield",
        "Identity",
        "Boolean",
        "Blob",
    ]);
    static ref NUMERIC_SCALAR_TYPES: HashSet<&'static str> = HashSet::from([
        "Int4",
        "Int8",
        "Int16",
        "UInt4",
        "UInt8",
        "UInt16",
        "Timestamp",
    ]);
    static ref STRING_SCALAR_TYPES: HashSet<&'static str> = HashSet::from([
        "ID",
        "Address",
        "AssetId",
        "Bytes4",
        "Bytes8",
        "Bytes32",
        "Bytes64",
        "Color",
        "ContractId",
        "Salt",
        "Json",
        "MessageId",
        "Charfield",
        "Identity",
        "Blob",
    ]);
    static ref SORTABLE_SCALAR_TYPES: HashSet<&'static str> = HashSet::from([
        "ID",
        "Address",
        "AssetId",
        "Int4",
        "Int8",
        "Int16",
        "UInt4",
        "UInt8",
        "UInt16",
        "Timestamp",
        "Color",
        "ContractId",
        "Salt",
        "MessageId",
        "Charfield",
        "Identity",
    ]);
}

const QUERY_ROOT: &str = "QueryRoot";

pub async fn execute_query(
    dynamic_request: Request,
    dynamic_schema: DynamicSchema,
    user_query: String,
) -> Result<Value, GraphqlError> {
    let raw_response = dynamic_schema
        .execute(dynamic_request.data(user_query))
        .await;

    // Unfortunately, due to the constraints of both our current process and async-graphql,
    // the following is a hack in order to return the correct response structure to the
    // user. In order to enable introspection that respects the user's schema,
    // fields had to be created; each field requires a FieldFuture to be created. To
    // ensure that the futures satisied the constraints of the library, I've chosen
    // to populate a fields with the results of the query. As such, the JSON response
    // needs to be modified so that the results are only included one time.
    let raw_json = match serde_json::to_value(raw_response) {
        Ok(json) => json,
        Err(e) => return Err(GraphqlError::DynamicQueryError(e.to_string())),
    };

    let mut data = match raw_json.get("data") {
        Some(val) => val.clone(),
        None => {
            return Err(GraphqlError::DynamicQueryError(
                "No key named 'data' in response".to_string(),
            ))
        }
    };

    let root_level_object_pointers = match data.as_object() {
        Some(obj) => obj
            .keys()
            .map(|k| format!("/{}", *k))
            .collect::<Vec<String>>(),
        None => {
            return Err(GraphqlError::DynamicQueryError(
                "Could not get object pointers for JSON response".to_string(),
            ))
        }
    };

    for root_lvl_obj_ptr in root_level_object_pointers {
        if let Some(unparsed_obj) = data.pointer(&root_lvl_obj_ptr) {
            if let Some(obj) = unparsed_obj.as_object() {
                let mut object_pointers = obj
                    .keys()
                    .map(|k| format!("{}/{}", root_lvl_obj_ptr, *k))
                    .collect::<Vec<String>>();

                while let Some(pointer) = object_pointers.pop() {
                    let val = match data.pointer(&pointer) {
                        Some(v) => v.clone(),
                        None => {
                            return Err(GraphqlError::DynamicQueryError(format!(
                                "Could not get value for {pointer}"
                            )))
                        }
                    };

                    // Found the results
                    if val.is_array() {
                        if let Some(data_obj) = data.pointer_mut(&root_lvl_obj_ptr) {
                            *data_obj = val.clone();
                        }
                        break;
                    } else if val.is_object() {
                        if let Some(obj) = val.as_object() {
                            let mut pointers = obj
                                .keys()
                                .map(|k| format!("{}/{}", pointer, *k))
                                .collect::<Vec<String>>();
                            object_pointers.append(&mut pointers);
                        }
                    } else {
                        continue;
                    }
                }
            }
        }
    }

    Ok(serde_json::json!({ "data": data }))
}

/// Build a dynamic schema. This allows for introspection, which allows for extensive
/// auto-documentation and code suggestions.
pub async fn build_dynamic_schema(
    schema: Schema,
    pool: IndexerConnectionPool,
) -> Result<DynamicSchema, GraphqlError> {
    let indexer_schema = parse_schema(schema.schema.clone())?;

    let result_cache: Option<ConstValue> = None;
    let query_executed: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));

    let mut schema_builder: DynamicSchemaBuilder =
        DynamicSchema::build(QUERY_ROOT, None, None)
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
            .data(pool)
            .data(query_executed)
            .data(result_cache);

    let mut input_objects = Vec::new();
    let mut filter_object_list = Vec::new();
    let mut filter_tracker = HashMap::new();
    let mut sort_object_list = Vec::new();
    let mut sorter_tracker = HashMap::new();

    let mut query_root_obj_type: Option<&ObjectType> = None;
    let sort_enum = Enum::new("SortOrder").item("asc").item("desc");

    for def in indexer_schema.definitions.iter() {
        match def {
            TypeSystemDefinition::Schema(_) => {}
            TypeSystemDefinition::Type(type_def_wrapper) => {
                let type_def = &type_def_wrapper.node;

                match &type_def.kind {
                    TypeKind::Scalar => todo!(),
                    TypeKind::Object(obj_type) => {
                        let obj_name = type_def.name.node.as_str();

                        // We want to do the query root last so that we can construct it
                        // with all of the type information for introspection.
                        if obj_name == QUERY_ROOT {
                            query_root_obj_type = Some(obj_type);
                        } else {
                            // Input values are stored for each field so that we can create
                            // a final input object for filtering and sorting a field.
                            let mut filter_input_vals = Vec::new();
                            let mut sort_input_vals = Vec::new();

                            // Field names will be added to this enum in order to allow
                            // for filtering on the column itself, i.e. "has" operator.
                            let mut object_field_enum =
                                Enum::new(format!("{obj_name}Fields"));

                            // Iterate through the fields to create input values and objects
                            // to construct field arguments.
                            for field_def in obj_type.fields.iter() {
                                let field_name = field_def.node.name.to_string();
                                let base_field_type = &field_def.node.ty.node.base;

                                if let BaseType::Named(field_type) = base_field_type {
                                    let (field_filter_input_val, mut field_input_objects) =
                                        create_filter_val_and_objects_for_field(
                                            &field_name,
                                            field_type.as_str(),
                                            obj_name,
                                        );

                                    filter_input_vals.push(field_filter_input_val);
                                    input_objects.append(&mut field_input_objects);

                                    if SORTABLE_SCALAR_TYPES.contains(field_type.as_str())
                                    {
                                        let sort_input_val = InputValue::new(
                                            field_name.clone(),
                                            TypeRef::named(sort_enum.type_name()),
                                        );
                                        sort_input_vals.push(sort_input_val);
                                    }
                                }

                                object_field_enum = object_field_enum.item(field_name);
                            }

                            let filter_object = filter_input_vals
                                .into_iter()
                                .fold(
                                    InputObject::new(format!("{obj_name}Filter")),
                                    |input_obj, input_val| input_obj.field(input_val),
                                )
                                .field(InputValue::new(
                                    "has",
                                    TypeRef::named_nn_list(object_field_enum.type_name()),
                                ));

                            let sort_object = sort_input_vals.into_iter().fold(
                                InputObject::new(format!("{obj_name}Sort")),
                                |input_obj, input_val| input_obj.field(input_val),
                            );

                            // For some reason, async-graphql does not implement the Hash
                            // trait on any of the type that we need for dynamic schemas.
                            // So we are essentially making a hash table ourselves for
                            // the filter and sort objects.
                            filter_object_list.push(filter_object);
                            filter_tracker.insert(
                                obj_name.to_string(),
                                filter_object_list.len() - 1,
                            );

                            sort_object_list.push(sort_object);
                            sorter_tracker
                                .insert(obj_name.to_string(), sort_object_list.len() - 1);

                            // Additionally, because we cannot refer to the object fields directly and
                            // associate the field arguments to them, we iterate through the fields a
                            // second time and construct the fields for the dynamic schema and add the
                            // field arguments as well.
                            let mut fields = Vec::new();
                            for field_def in obj_type.fields.iter() {
                                let field_name = field_def.node.name.to_string();
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

                                let schema = Arc::new(schema.clone());

                                let mut field = Field::new(
                                    field_name.clone(),
                                    field_type.clone(),
                                    move |ctx: ResolverContext| {
                                        let schema = schema.clone();
                                        return FieldFuture::new(async move {
                                            Ok(Some(
                                                query_for_results(schema, &ctx).await?,
                                            ))
                                        });
                                    },
                                );

                                if let BaseType::Named(field_type) = base_field_type {
                                    if !SCALAR_TYPES.contains(field_type.as_str()) {
                                        if let Some(idx) =
                                            filter_tracker.get(&field_type.to_string())
                                        {
                                            let object_filter_arg = InputValue::new(
                                                "filter",
                                                TypeRef::named(
                                                    filter_object_list[*idx].type_name(),
                                                ),
                                            );
                                            field = field.argument(object_filter_arg)
                                        }

                                        if let Some(idx) =
                                            sorter_tracker.get(&field_type.to_string())
                                        {
                                            let object_sort_arg = InputValue::new(
                                                "order",
                                                TypeRef::named(
                                                    sort_object_list[*idx].type_name(),
                                                ),
                                            );
                                            field = field.argument(object_sort_arg);
                                        }

                                        let offset_arg = InputValue::new(
                                            "offset",
                                            TypeRef::named(TypeRef::INT),
                                        );

                                        let limit_arg = InputValue::new(
                                            "first",
                                            TypeRef::named(TypeRef::INT),
                                        );

                                        let id_selection_arg = InputValue::new(
                                            "id",
                                            TypeRef::named(TypeRef::STRING),
                                        );

                                        field = field
                                            .argument(offset_arg)
                                            .argument(limit_arg)
                                            .argument(id_selection_arg);
                                    }
                                }

                                fields.push(field);
                            }

                            let obj = fields
                                .into_iter()
                                .fold(Object::new(obj_name), |obj, f| obj.field(f));
                            schema_builder =
                                schema_builder.register(obj).register(object_field_enum);
                        }
                    }
                    TypeKind::Interface(_) => todo!(),
                    TypeKind::Union(_) => todo!(),
                    TypeKind::Enum(_) => todo!(),
                    TypeKind::InputObject(_) => todo!(),
                }
            }
            TypeSystemDefinition::Directive(_) => {}
        }
    }

    // Now that all of the other fields have been constructed, we can ensure that
    // the fields used in the root query object will have type information available
    // during introspection.
    if let Some(obj_type) = query_root_obj_type {
        let mut fields = Vec::new();
        for field_def in obj_type.fields.iter() {
            let field_name = field_def.node.name.to_string();
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

            let schema = Arc::new(schema.clone());

            let mut field = Field::new(
                field_name.clone(),
                field_type.clone(),
                move |ctx: ResolverContext| {
                    let schema = schema.clone();
                    return FieldFuture::new(async move {
                        Ok(Some(query_for_results(schema, &ctx).await?))
                    });
                },
            );

            if let BaseType::Named(field_type) = base_field_type {
                if !SCALAR_TYPES.contains(field_type.as_str()) {
                    if let Some(idx) = filter_tracker.get(&field_type.to_string()) {
                        let object_filter_arg = InputValue::new(
                            "filter",
                            TypeRef::named(filter_object_list[*idx].type_name()),
                        );
                        field = field.argument(object_filter_arg)
                    }

                    if let Some(idx) = sorter_tracker.get(&field_type.to_string()) {
                        let sort_obj_arg = InputValue::new(
                            "order",
                            TypeRef::named(sort_object_list[*idx].type_name()),
                        );
                        field = field.argument(sort_obj_arg)
                    }

                    let offset_arg =
                        InputValue::new("offset", TypeRef::named(TypeRef::INT));

                    let limit_arg =
                        InputValue::new("first", TypeRef::named(TypeRef::INT));

                    let id_selection_arg =
                        InputValue::new("id", TypeRef::named(TypeRef::INT));

                    field = field
                        .argument(offset_arg)
                        .argument(limit_arg)
                        .argument(id_selection_arg);
                }
            }

            fields.push(field);
        }

        // TODO: Add PageInfo field

        // TODO: Add nodes and edges?

        let query_root = fields
            .into_iter()
            .fold(Object::new(QUERY_ROOT), |obj, f| obj.field(f));
        schema_builder = schema_builder.register(query_root);
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

    match schema_builder.finish() {
        Ok(schema) => Ok(schema),
        Err(e) => Err(GraphqlError::DynamicSchemaBuildError(e)),
    }
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

    // TODO: Add AND, OR, and NOT

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

/// Retrieve query results. If the query has not yet been executed, the database
/// will be queried and the results will be cached internally. If the query has
/// already been executed, the results are returned from the cache.
async fn query_for_results<'a>(
    schema: Arc<Schema>,
    ctx: &ResolverContext<'_>,
) -> Result<FieldValue<'a>, GraphqlError> {
    // A mutex is used because field resolvers are executed concurrently. This
    // ensures that a database query is run only once.
    let mut executed = match ctx.data::<Arc<Mutex<bool>>>() {
        Ok(mtx) => mtx.lock().await,
        Err(_) => {
            return Err(GraphqlError::DynamicQueryError(
                "Unable to retrieve lock from context".to_string(),
            ))
        }
    };

    if !(*executed) {
        let pool = match ctx.data::<IndexerConnectionPool>() {
            Ok(p) => p,
            Err(_) => {
                return Err(GraphqlError::DynamicQueryError(
                    "Unable to retrieve pool from context".to_string(),
                ))
            }
        };

        let user_query = match ctx.data::<String>() {
            Ok(uq) => uq,
            Err(_) => {
                return Err(GraphqlError::DynamicQueryError(
                    "Unable to retrieve user query from context".to_string(),
                ))
            }
        };

        let query = GraphqlQueryBuilder::new(&schema, user_query)?.build()?;

        let queries = query.as_sql(&schema, pool.database_type())?.join(";\n");
        let mut conn = match pool.acquire().await {
            Ok(c) => c,
            Err(e) => return Err(GraphqlError::DynamicQueryError(e.to_string())),
        };

        let results = match queries::run_query(&mut conn, queries).await {
            Ok(val) => val,
            Err(e) => return Err(GraphqlError::DynamicQueryError(e.to_string())),
        };

        let stored_results = &mut ctx.data::<Option<ConstValue>>();

        let stored_results = match stored_results {
            Ok(sr) => sr,
            Err(_) => {
                return Err(GraphqlError::DynamicQueryError(
                    "Unable to retrieve cached query results".to_string(),
                ))
            }
        };

        let parsed_results = match ConstValue::from_json(results) {
            Ok(cv) => cv,
            Err(_) => {
                return Err(GraphqlError::DynamicQueryError(
                    "Unable to parse query results from JSON".to_string(),
                ))
            }
        };
        *stored_results = &Some(parsed_results.clone());

        *executed = true;

        Ok(FieldValue::from(parsed_results))
    } else if let Ok(Some(cached_results)) = ctx.data::<Option<ConstValue>>() {
        Ok(FieldValue::from(cached_results.clone()))
    } else {
        match ctx.parent_value.as_value() {
            Some(results_from_parent) => {
                Ok(FieldValue::from(results_from_parent.clone()))
            }
            None => Err(GraphqlError::DynamicSchemaBuildError(SchemaError::from(
                "No results available".to_string(),
            ))),
        }
    }
}

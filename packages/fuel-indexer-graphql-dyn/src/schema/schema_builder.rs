use super::data::*;
use super::loader::*;
use super::node::*;
use super::resolver::*;
use super::resolver_context::*;
use super::schema_type::*;
use super::self_prelude::*;

pub struct DynSchemaBuilder {
    schema: SchemaBuilder,
    query: Object,
}

impl Default for DynSchemaBuilder {
    fn default() -> Self {
        let mut schema = Schema::build(TypeRef::QUERY, None, None);
        schema = schema
            .register(Object::new_page_info::<DynResolver>())
            .register(Interface::new_node());
        let query = Object::new_query::<DynResolver>();
        Self { schema, query }
    }
}

impl DynSchemaBuilder {
    pub fn new(schema_type: &DynSchemaType, store: Arc<Mutex<dyn store::Store>>) -> Self {
        let loader = DynLoader::new(store);
        let resolver = DynResolver::new(&schema_type, Arc::new(Mutex::new(loader)));
        let DynSchemaBuilder {
            mut schema,
            mut query,
        } = DynSchemaBuilder::default();

        for (_data_type_id, data_type) in &schema_type.data {
            match data_type {
                // GraphQL types
                DynDataType::ID
                | DynDataType::String
                | DynDataType::Int
                | DynDataType::Float
                | DynDataType::Boolean
                | DynDataType::List(_) => continue,
                // Fuel types
                DynDataType::Unit
                | DynDataType::U8
                | DynDataType::U16
                | DynDataType::U32
                | DynDataType::U64
                | DynDataType::B256
                | DynDataType::Bytes => {
                    let scalar = Scalar::new(data_type.name().to_pascal_string());
                    schema = schema.register(scalar);
                }
                DynDataType::Object(name, fields) => {
                    let mut object = Object::new(name.to_pascal_string());

                    for (_field_id, field_type) in fields {
                        object = object.field(Field::new(
                            field_type.name.to_camel_string(),
                            TypeRef::named_nn(field_type.data_type_id().to_string()),
                            |_ctx| todo!(),
                        ))
                    }

                    schema = schema.register(object);
                }
                DynDataType::Enum(name, variants) => {
                    if variants
                        .iter()
                        .any(|(_, variant)| variant.data_type_id != DynDataTypeId::Unit)
                    {
                        let mut union = Union::new(name.to_pascal_string());

                        for (_variant_id, variant_type) in variants {
                            union =
                                union.possible_type(variant_type.name.to_pascal_string())
                        }

                        schema = schema.register(union);
                    } else {
                        let mut r#enum = Enum::new(name.to_pascal_string());

                        for (_variant_id, variant_type) in variants {
                            r#enum = r#enum
                                .item(EnumItem::new(variant_type.name.to_pascal_string()))
                        }

                        schema = schema.register(r#enum);
                    }
                }
            }
        }

        for (_node_type_id, node_type) in &schema_type.node {
            let mut node =
                Object::new_node::<DynResolver>(&node_type.name.to_pascal_string());

            for (_field_id, field_type) in &node_type.fields {
                match field_type {
                    DynNodeFieldType::Data(field_type) => {
                        node = node.data_field::<DynResolver>(
                            field_type.name.to_camel_string(),
                            TypeRef::named_nn(field_type.data_type_id().to_string()),
                        );
                    }
                    DynNodeFieldType::Ref(field_type) => {
                        node = node.ref_field::<DynResolver>(
                            field_type.name.to_camel_string(),
                            TypeRef::named_nn(field_type.ref_node_type_id.to_string()),
                        );
                    }
                    DynNodeFieldType::Connection(field_type) => {
                        node = node.connection_field::<DynResolver>(
                            field_type.name.to_camel_string(),
                            TypeRef::named_nn(TypeRef::connection(
                                field_type.edge_type_id(),
                            )),
                        );
                    }
                }
            }

            schema = schema.register(node);
        }

        for (edge_type_id, edge_type) in &schema_type.edge {
            let connection = Object::new_connection::<DynResolver>(
                edge_type_id.clone(),
                edge_type.head_type_id().clone(),
            );
            let connection_edge = Object::new_connection_edge::<DynResolver>(
                edge_type_id.clone(),
                TypeRef::node(edge_type.head_type_id().clone()),
            );

            schema = schema.register(connection).register(connection_edge);
        }

        schema = schema.data(resolver);

        for (_node_type_id, node_type) in &schema_type.node {
            query = query.field(
                Field::new(
                    node_type.name.to_camel_string(),
                    TypeRef::named(TypeRef::node(node_type.name.to_pascal_string())),
                    move |ctx| {
                        FieldFuture::new(async move {
                            let id = ctx.get_arg_string_nn("id")?;

                            let resolver = ctx.resolver();
                            let loader = resolver.locked_loader().await;
                            let node =
                                loader.load_node_one(&id.to_string()).await.unwrap();
                            if let Some(node) = node {
                                // Ok(Some(FieldValue::value(node.clone())))
                                Ok(Some(FieldValue::owned_any(node.clone())))
                            } else {
                                Ok(None)
                            }
                        })
                    },
                )
                .node_by_id_arguments(),
            );
        }

        Self { schema, query }
    }

    pub fn finish(self) -> Result<Schema, ()> {
        let Self { schema, query } = self;
        let schema = schema.register(query);
        Ok(schema.finish().unwrap())
    }
}

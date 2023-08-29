use super::connection::*;
use super::data::*;
use super::loader::*;
use super::node::*;
use super::paging::*;
use super::resolver_context::*;
use super::schema_type::*;
use super::self_prelude::*;

pub struct DynResolver {
    schema_type: DynSchemaType,
    loader: Arc<Mutex<DynLoader>>,
}

#[derive(thiserror::Error, Debug)]
pub enum DynResolverError {
    #[error(transparent)]
    ResolverContext(#[from] DynResolverContextError),
    #[error(transparent)]
    Loader(#[from] DynLoaderError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type DynResolverResult<T> = anyhow::Result<T, DynResolverError>;

pub type DynResolverFn<'a> =
    Box<dyn Fn(ResolverContext<'a>) -> FieldFuture<'a> + Send + Sync + 'static>;

impl DynResolver {
    pub fn new(schema_type: &DynSchemaType, loader: Arc<Mutex<DynLoader>>) -> Self {
        Self {
            schema_type: schema_type.clone(),
            loader,
        }
    }

    pub async fn locked_loader(&self) -> tokio::sync::MutexGuard<'_, DynLoader> {
        self.loader.lock().await
    }

    //
    pub fn resolve_node_id_arg(&self, arg: ValueAccessor) -> DynNodeId {
        let id = arg.string().unwrap();
        let (input_type, input_id) = id.split_once(':').unwrap();
        // let node_type = self.get_node_type(input_type.to_string());
        let id = format!("{}:{}", input_type, input_id);
        id
    }
    pub fn resolve_node_impl_id(
        &self,
        node_type_id: &DynNodeTypeId,
        input: &str,
    ) -> DynNodeId {
        // unimplemented!()
        let (input_type, input_id) = input.split_once(':').unwrap();
        let input_type: DynNodeTypeId = input_type.parse().unwrap();
        if &input_type != node_type_id {
            panic!("invalid node id: {}", input);
        }
        let id = format!("{}:{}", input_type, input_id);
        id
    }

    // Field type methods
    // pub fn get_node_data_field_type(
    //     &self,
    //     id: &ObjTypeId,
    //     field_id: &DynamicFieldId,
    // ) -> &DynamicObjectTypeDataField {
    //     let node_type = self.get_node_type(id);
    //     node_type
    //         .data
    //         .iter()
    //         .find(|field_type| field_type.id() == field_id)
    //         .unwrap()
    // }
    // pub fn get_node_ref_field_type(
    //     &self,
    //     id: &ObjTypeId,
    //     field_id: &DynamicFieldId,
    // ) -> &DynamicNodeTypeRefField {
    //     let node_type = self.get_node_type(id);
    //     node_type
    //         .ref_fields
    //         .iter()
    //         .find(|field_type| field_type.id() == field_id)
    //         .unwrap()
    // }
    // pub fn get_node_connection_field_type(
    //     &self,
    //     id: &ObjTypeId,
    //     field_id: &DynamicFieldId,
    // ) -> &DynamicNodeTypeConnectionField {
    //     let node_type = self.get_node_type(id);
    //     node_type
    //         .connection_fields
    //         .iter()
    //         .find(|field_type| field_type.id() == field_id)
    //         .unwrap()
    // }
}

impl QueryResolver for DynResolver {
    fn resolve_node_by_id(ctx: ResolverContext) -> FieldFuture {
        FieldFuture::new(async move {
            let resolver = ctx.resolver();
            let id = resolver.resolve_node_id_arg(ctx.args.get("id").unwrap());

            let loader = resolver.loader.lock().await;
            let node = loader.load_node_one(&id).await.unwrap();
            if let Some(node) = node {
                let type_id = node.type_id().to_string();
                Ok(Some(FieldValue::owned_any(node.clone()).with_type(type_id)))
            } else {
                Ok(None)
            }
        })
    }
    fn resolve_nodes_by_id(ctx: ResolverContext) -> FieldFuture {
        FieldFuture::new(async move {
            let ids = ctx.get_arg_list_nn_node_id_nn("ids")?;

            let resolver = ctx.resolver();
            let loader = resolver.loader.lock().await;
            let nodes = loader.load_node_many(ids.as_slice()).await.unwrap();
            let nodes = nodes
                .into_iter()
                .map(|node| {
                    if let Some(node) = node {
                        let type_id = node.type_id().to_string();
                        FieldValue::owned_any(node.clone()).with_type(type_id)
                    } else {
                        FieldValue::NULL
                    }
                })
                .collect::<Vec<_>>();
            Ok(Some(FieldValue::list(nodes)))
        })
    }
}

impl NodeResolver for DynResolver {
    fn resolve_id(
        ctx: async_graphql::dynamic::ResolverContext,
    ) -> async_graphql::dynamic::FieldFuture {
        FieldFuture::new(async move {
            let parent = ctx.parent_node()?;

            let value: String = parent.id().to_string();
            Ok(Some(FieldValue::value(value)))
        })
    }
    fn resolve_data(ctx: ResolverContext) -> FieldFuture {
        FieldFuture::new(async move {
            let parent = ctx.parent_node()?;
            let field_name = ctx.field_name();

            let resolver = ctx.resolver();
            let field_type = resolver
                .schema_type
                .node_field(&parent.type_id(), &field_name)
                .as_data_field();

            let data_type = resolver.schema_type.data(field_type.data_type_id());

            let field_value = &parent.data()[field_type.store_id()];

            match data_type {
                DynDataType::ID => {
                    let value = field_value.as_str().unwrap();
                    Ok(Some(FieldValue::value(value)))
                }
                DynDataType::String => {
                    let value = field_value.as_str().unwrap();
                    Ok(Some(FieldValue::value(value)))
                }
                DynDataType::Int => {
                    let value = field_value.as_i64().unwrap();
                    Ok(Some(FieldValue::value(value)))
                }
                DynDataType::Float => {
                    let value = field_value.as_f64().unwrap();
                    Ok(Some(FieldValue::value(value)))
                }
                DynDataType::List(_ty) => {
                    todo!();
                }
                DynDataType::Unit => Ok(Some(FieldValue::NULL)),
                DynDataType::Boolean => {
                    let value = field_value.as_bool().unwrap();
                    Ok(Some(FieldValue::value(value)))
                }
                DynDataType::U8 => {
                    let value = field_value.as_u64().unwrap() as u8;
                    Ok(Some(FieldValue::value(value)))
                }
                DynDataType::U16 => {
                    let value = field_value.as_u64().unwrap() as u16;
                    Ok(Some(FieldValue::value(value)))
                }
                DynDataType::U32 => {
                    let value = field_value.as_u64().unwrap() as u32;
                    Ok(Some(FieldValue::value(value)))
                }
                DynDataType::U64 => {
                    let value = field_value.as_u64().unwrap();
                    Ok(Some(FieldValue::value(value)))
                }
                DynDataType::B256 => {
                    let value = field_value.as_str().unwrap();
                    Ok(Some(FieldValue::value(value)))
                }
                DynDataType::Bytes => {
                    let value = field_value.as_str().unwrap();
                    Ok(Some(FieldValue::value(value)))
                }
                DynDataType::Object(_name, _fields) => {
                    todo!()
                }
                DynDataType::Enum(_name, _variants) => {
                    todo!()
                }
            }
        })
    }
    fn resolve_ref(ctx: ResolverContext) -> FieldFuture {
        FieldFuture::new(async move {
            let parent = ctx.parent_node()?;
            let field_name = ctx.field_name();

            let resolver = ctx.resolver();
            let field_type = resolver
                .schema_type
                .node_field(&parent.type_id(), &field_name)
                .as_ref_field();

            let data_field_id = &field_type.store_id;
            let id = parent.data().get(data_field_id).ok_or_else(|| {
                DynResolverError::Other(anyhow!(
                    "invalid data field id: {}",
                    data_field_id
                ))
            })?;
            let id = id.as_str().unwrap();
            let resolver = ctx.resolver();
            let loader = resolver.loader.lock().await;
            let node = loader
                .load_node_one(&id.to_string())
                .await
                .unwrap()
                .unwrap();
            Ok(Some(FieldValue::owned_any(node.clone())))
        })
    }
    fn resolve_connection(ctx: ResolverContext) -> FieldFuture {
        FieldFuture::new(async move {
            let parent = ctx.parent_node()?;
            let paging = ctx.get_paging()?;
            let field_name = ctx.field().name().to_string();

            let resolver = ctx.resolver();
            let field_type = resolver
                .schema_type
                .node_field(&parent.type_id(), &field_name)
                .as_connection_field();

            let _node_type = resolver.schema_type.node(&parent.type_id());
            let edge_type_id = field_type.edge_type_id();
            let _edge_type = resolver.schema_type.edge(edge_type_id);
            let resolver = ctx.resolver();
            let loader = resolver.loader.lock().await;

            let all_edges = loader
                .load_node_edges(parent.id(), edge_type_id)
                .await
                .unwrap();

            let mut has_next_page = false;
            let mut has_previous_page = false;
            let mut edges = all_edges;
            match paging.direction {
                DynPagingDirection::Forward => {
                    let DynPaging {
                        count,
                        after,
                        before,
                        ..
                    } = paging;
                    edges = edges
                        .into_iter()
                        .skip_while(|edge| match &after {
                            Some(after) => {
                                if &edge.time().to_string() == after {
                                    false
                                } else {
                                    has_previous_page = true;
                                    true
                                }
                            }
                            None => false,
                        })
                        .take_while(|edge| match &before {
                            Some(before) => {
                                if &edge.time().to_string() == before {
                                    has_next_page = true;
                                    false
                                } else {
                                    true
                                }
                            }
                            None => true,
                        })
                        .take(count as usize)
                        .collect::<Vec<_>>();
                }
                DynPagingDirection::Backward => {
                    todo!()
                }
            }

            let edges = edges
                .iter()
                .map(|edge| {
                    DynConnectionEdge::new(edge.head_id(), &edge.time().to_string())
                })
                .collect::<Vec<_>>();
            let total_count = edges.len();
            let start_cursor = edges.first().map(|edge| edge.cursor().to_string());
            let end_cursor = edges.last().map(|edge| edge.cursor().to_string());
            let connection = DynConnection {
                edges,
                total_count,
                page_info: DynPageInfo {
                    has_next_page,
                    has_previous_page,
                    start_cursor,
                    end_cursor,
                },
            };
            Ok(Some(FieldValue::owned_any(connection)))
        })
    }
}

impl ConnectionResolver for DynResolver {
    fn resolve_total_count(ctx: ResolverContext) -> FieldFuture {
        FieldFuture::new(async move {
            let parent = ctx.parent::<DynConnection>()?;
            Ok(Some(FieldValue::value(parent.total_count)))
        })
    }
    fn resolve_nodes(ctx: ResolverContext) -> FieldFuture {
        FieldFuture::new(async move {
            let parent = ctx.parent::<DynConnection>()?;
            let node_ids = parent
                .edges
                .iter()
                .map(|edge| edge.node_id.clone())
                .collect::<Vec<_>>();
            let resolver = ctx.resolver();
            let loader = resolver.loader.lock().await;
            let nodes = loader.load_node_many(node_ids.as_slice()).await.unwrap();
            let nodes = nodes
                .into_iter()
                .map(|node| {
                    if let Some(node) = node {
                        FieldValue::owned_any(node.clone())
                    } else {
                        FieldValue::NULL
                    }
                })
                .collect::<Vec<_>>();
            Ok(Some(FieldValue::list(nodes)))
        })
    }
    fn resolve_edges(ctx: ResolverContext) -> FieldFuture {
        FieldFuture::new(async move {
            let parent = ctx.parent::<DynConnection>()?;
            let edges = parent
                .edges
                .iter()
                .map(|edge| FieldValue::owned_any(edge.clone()))
                .collect::<Vec<_>>();
            Ok(Some(FieldValue::list(edges)))
        })
    }
    fn resolve_page_info(ctx: ResolverContext) -> FieldFuture {
        FieldFuture::new(async move {
            let parent = ctx.parent::<DynConnection>()?;
            Ok(Some(FieldValue::owned_any(parent.page_info.clone())))
        })
    }
}

impl ConnectionEdgeResolver for DynResolver {
    fn resolve_node(ctx: ResolverContext) -> FieldFuture {
        FieldFuture::new(async move {
            let parent = ctx.parent::<DynConnectionEdge>()?;
            let id = &parent.node_id;
            let resolver = ctx.resolver();
            let loader = resolver.loader.lock().await;
            let node = loader.load_node_one(id).await.unwrap().unwrap();
            Ok(Some(FieldValue::owned_any(node.clone())))
        })
    }
    fn resolve_cursor(ctx: ResolverContext) -> FieldFuture {
        FieldFuture::new(async move {
            let parent = ctx.parent::<DynConnectionEdge>()?;
            Ok(Some(FieldValue::value(parent.cursor.clone())))
        })
    }
}

impl PageInfoResolver for DynResolver {
    fn resolve_has_next_page(
        ctx: async_graphql::dynamic::ResolverContext,
    ) -> async_graphql::dynamic::FieldFuture {
        FieldFuture::new(async move {
            let parent = ctx.parent::<DynPageInfo>()?;
            Ok(Some(FieldValue::value(parent.has_next_page)))
        })
    }
    fn resolve_has_previous_page(ctx: ResolverContext) -> FieldFuture {
        FieldFuture::new(async move {
            let parent = ctx.parent::<DynPageInfo>()?;
            Ok(Some(FieldValue::value(parent.has_previous_page)))
        })
    }
    fn resolve_start_cursor(ctx: ResolverContext) -> FieldFuture {
        FieldFuture::new(async move {
            let parent = ctx.parent::<DynPageInfo>()?;
            if let Some(start_cursor) = &parent.start_cursor {
                Ok(Some(FieldValue::value(start_cursor.clone())))
            } else {
                Ok(None)
            }
        })
    }
    fn resolve_end_cursor(ctx: ResolverContext) -> FieldFuture {
        FieldFuture::new(async move {
            let parent = ctx.parent::<DynPageInfo>()?;
            if let Some(end_cursor) = &parent.end_cursor {
                Ok(Some(FieldValue::value(end_cursor.clone())))
            } else {
                Ok(None)
            }
        })
    }
}

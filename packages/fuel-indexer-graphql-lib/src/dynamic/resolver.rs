use super::connection::*;
use super::data::*;
use super::loader::*;
use super::node::*;
use super::paging::*;
use super::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct DynamicResolver {
    pub node_types: HashMap<DynamicNodeTypeId, DynamicNodeType>,
    pub loader: Arc<Mutex<dyn DynamicLoader>>,
}
impl DynamicResolver {
    pub fn new(
        node_types: Vec<DynamicNodeType>,
        loader: Arc<Mutex<dyn DynamicLoader>>,
    ) -> Self {
        let node_types = node_types
            .into_iter()
            .map(|node_type| (node_type.id.clone(), node_type))
            .collect::<HashMap<_, _>>();
        Self { node_types, loader }
    }
}

impl QueryResolver for DynamicResolver {
    fn resolve_node_by_id(ctx: ResolverContext) -> FieldFuture {
        FieldFuture::new(async move {
            let resolver = ctx.data_unchecked::<Self>();
            let id = ctx.args.get("id").unwrap();
            let id = id.string().unwrap();
            let id: DynamicNodeId = id.parse().unwrap();
            let loader = resolver.loader.lock().await;
            let node = loader.load_node_by_id(&id).await.unwrap();
            if let Some(node) = node {
                let type_id = node.type_id().clone();
                Ok(Some(FieldValue::owned_any(node.clone()).with_type(type_id)))
            } else {
                Ok(None)
            }
        })
    }
    fn resolve_node_by_ids(ctx: ResolverContext) -> FieldFuture {
        FieldFuture::new(async move {
            let resolver = ctx.data_unchecked::<Self>();
            let ids = ctx.args.get("ids").unwrap();
            let ids = ids.list().unwrap();
            let ids = ids
                .iter()
                .map(|id| id.string().unwrap().parse())
                .collect::<Result<Vec<DynamicNodeId>, _>>()
                .unwrap();
            let loader = resolver.loader.lock().await;
            let nodes = loader.load_nodes_by_id(ids.as_slice()).await.unwrap();
            let nodes = nodes
                .into_iter()
                .map(|node| {
                    if let Some(node) = node {
                        let type_id = node.type_id().clone();
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

impl NodeResolver for DynamicResolver {
    fn resolve_id(
        ctx: async_graphql::dynamic::ResolverContext,
    ) -> async_graphql::dynamic::FieldFuture {
        FieldFuture::new(async move {
            let parent = ctx.parent_value.try_downcast_ref::<DynamicNode>()?;
            let value: String = parent.id().into();
            Ok(Some(FieldValue::value(value)))
        })
    }
    fn resolve_data(ctx: ResolverContext) -> FieldFuture {
        FieldFuture::new(async move {
            let parent = ctx.parent_value.try_downcast_ref::<DynamicNode>()?;
            let path_node = ctx.path_node.unwrap();
            let field_name = path_node.field_name();
            let resolver = ctx.data_unchecked::<Self>();
            let node_type = resolver.node_types.get(parent.type_id()).unwrap();
            let field_type = node_type
                .data_fields
                .iter()
                .find(|field_type| field_type.id() == field_name)
                .unwrap();
            match field_type.data_type() {
                DynamicDataType::String => {
                    let value = parent.data()[field_name].as_str().unwrap();
                    Ok(Some(FieldValue::value(value)))
                }
                DynamicDataType::Int => {
                    let value = parent.data()[field_name].as_i64().unwrap();
                    Ok(Some(FieldValue::value(value)))
                }
            }
        })
    }
    fn resolve_ref(ctx: ResolverContext) -> FieldFuture {
        FieldFuture::new(async move {
            let parent = ctx.parent_value.try_downcast_ref::<DynamicNode>()?;
            let path_node = ctx.path_node.unwrap();
            let field_name = path_node.field_name();
            let resolver = ctx.data_unchecked::<Self>();
            let node_type = resolver.node_types.get(parent.type_id()).unwrap();
            let field_type = node_type
                .ref_fields
                .iter()
                .find(|field_type| field_type.id == field_name)
                .unwrap();
            let data_field_id = &field_type.data_field_id;
            let local_id = parent.data()[data_field_id].as_str().unwrap();
            let id = DynamicNodeId::new(field_type.ref_node_type_id.clone(), local_id);
            let resolver = ctx.data_unchecked::<Self>();
            let loader = resolver.loader.lock().await;
            let node = loader.load_node_by_id(&id).await.unwrap().unwrap();
            Ok(Some(FieldValue::owned_any(node.clone())))
        })
    }
    fn resolve_connection(ctx: ResolverContext) -> FieldFuture {
        FieldFuture::new(async move {
            let parent = ctx.parent_value.try_downcast_ref::<DynamicNode>()?;
            let path_node = ctx.path_node.unwrap();
            let field_name = path_node.field_name();
            let resolver = ctx.data_unchecked::<Self>();
            let node_type = resolver.node_types.get(parent.type_id()).unwrap();
            let field_type = node_type
                .connection_fields
                .iter()
                .find(|field_type| field_type.id == field_name)
                .unwrap();
            let resolver = ctx.data_unchecked::<Self>();
            match (node_type.id.as_str(), field_type.ref_node_type_id.as_str()) {
                ("Chain", "Block") => {
                    let loader = resolver.loader.lock().await;
                    let edges = loader
                        .load_edges(&"ChainHasBlock".to_string(), parent.local_id())
                        .await
                        .unwrap();
                    let edges = edges
                        .iter()
                        .map(|edge| DynamicConnectionEdge {
                            node_id: DynamicNodeId::new("Block", edge.head_local_id()),
                            cursor: "".to_string(),
                        })
                        .collect::<Vec<_>>();
                    let total_count = edges.len();
                    let connection = DynamicConnection {
                        edges,
                        total_count,
                        page_info: DynamicPageInfo {
                            has_next_page: false,
                            has_previous_page: false,
                            start_cursor: None,
                            end_cursor: None,
                        },
                    };
                    Ok(Some(FieldValue::owned_any(connection)))
                }
                ("Chain", "Transaction") => {
                    let loader = resolver.loader.lock().await;
                    let edges = loader
                        .load_edges(&"ChainHasTransaction".to_string(), parent.local_id())
                        .await
                        .unwrap();
                    let edges = edges
                        .iter()
                        .map(|edge| DynamicConnectionEdge {
                            node_id: DynamicNodeId::new(
                                "Transaction",
                                edge.head_local_id(),
                            ),
                            cursor: "".to_string(),
                        })
                        .collect::<Vec<_>>();
                    let total_count = edges.len();
                    let connection = DynamicConnection {
                        edges,
                        total_count,
                        page_info: DynamicPageInfo {
                            has_next_page: false,
                            has_previous_page: false,
                            start_cursor: None,
                            end_cursor: None,
                        },
                    };
                    Ok(Some(FieldValue::owned_any(connection)))
                }
                _ => unreachable!(),
            }
        })
    }
}

impl ConnectionResolver for DynamicResolver {
    fn resolve_total_count(ctx: ResolverContext) -> FieldFuture {
        FieldFuture::new(async move {
            let parent = ctx.parent_value.try_downcast_ref::<DynamicConnection>()?;
            Ok(Some(FieldValue::value(parent.total_count)))
        })
    }
    fn resolve_nodes(ctx: ResolverContext) -> FieldFuture {
        FieldFuture::new(async move {
            let parent = ctx.parent_value.try_downcast_ref::<DynamicConnection>()?;
            let node_ids = parent
                .edges
                .iter()
                .map(|edge| edge.node_id.clone())
                .collect::<Vec<_>>();
            let resolver = ctx.data_unchecked::<Self>();
            let loader = resolver.loader.lock().await;
            let nodes = loader.load_nodes_by_id(node_ids.as_slice()).await.unwrap();
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
            let parent = ctx.parent_value.try_downcast_ref::<DynamicConnection>()?;
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
            let parent = ctx.parent_value.try_downcast_ref::<DynamicConnection>()?;
            Ok(Some(FieldValue::owned_any(parent.page_info.clone())))
        })
    }
}

impl EdgeResolver for DynamicResolver {
    fn resolve_node(ctx: ResolverContext) -> FieldFuture {
        FieldFuture::new(async move {
            let parent = ctx
                .parent_value
                .try_downcast_ref::<DynamicConnectionEdge>()?;
            let id = &parent.node_id;
            let resolver = ctx.data_unchecked::<Self>();
            let loader = resolver.loader.lock().await;
            let node = loader.load_node_by_id(id).await.unwrap().unwrap();
            Ok(Some(FieldValue::owned_any(node.clone())))
        })
    }
    fn resolve_cursor(ctx: ResolverContext) -> FieldFuture {
        FieldFuture::new(async move {
            let parent = ctx
                .parent_value
                .try_downcast_ref::<DynamicConnectionEdge>()?;
            Ok(Some(FieldValue::value(parent.cursor.clone())))
        })
    }
}

impl PageInfoResolver for DynamicResolver {
    fn resolve_has_next_page(
        ctx: async_graphql::dynamic::ResolverContext,
    ) -> async_graphql::dynamic::FieldFuture {
        FieldFuture::new(async move {
            let parent = ctx.parent_value.try_downcast_ref::<DynamicPageInfo>()?;
            Ok(Some(FieldValue::value(parent.has_next_page)))
        })
    }
    fn resolve_has_previous_page(ctx: ResolverContext) -> FieldFuture {
        FieldFuture::new(async move {
            let parent = ctx.parent_value.try_downcast_ref::<DynamicPageInfo>()?;
            Ok(Some(FieldValue::value(parent.has_previous_page)))
        })
    }
    fn resolve_start_cursor(ctx: ResolverContext) -> FieldFuture {
        FieldFuture::new(async move {
            let parent = ctx.parent_value.try_downcast_ref::<DynamicPageInfo>()?;
            if let Some(start_cursor) = &parent.start_cursor {
                Ok(Some(FieldValue::value(start_cursor.clone())))
            } else {
                Ok(None)
            }
        })
    }
    fn resolve_end_cursor(ctx: ResolverContext) -> FieldFuture {
        FieldFuture::new(async move {
            let parent = ctx.parent_value.try_downcast_ref::<DynamicPageInfo>()?;
            if let Some(end_cursor) = &parent.end_cursor {
                Ok(Some(FieldValue::value(end_cursor.clone())))
            } else {
                Ok(None)
            }
        })
    }
}

use std::collections::HashMap;

pub use crate::spec::*;

pub(self) mod prelude {
    pub use crate::spec::*;
    pub use async_graphql::dynamic::*;
    pub use serde_json::Value as JsonValue;
}

use prelude::*;

pub struct Node(pub String, pub JsonValue);
impl Node {
    pub fn id(&self) -> &str {
        &self.0
    }
    pub fn type_id(&self) -> &str {
        self.id().split(':').next().unwrap()
    }
    pub fn local_id(&self) -> &str {
        self.id().split(':').nth(1).unwrap()
    }
    pub fn data(&self) -> &JsonValue {
        &self.1
    }

    pub fn to_field_value(&self) -> FieldValue {
        FieldValue::borrowed_any(self)
    }
    pub fn to_typed_field_value(&self) -> FieldValue {
        let type_id = self.type_id().to_string();
        FieldValue::borrowed_any(self).with_type(type_id)
    }
}

pub struct JsonResolver {
    pub nodes: HashMap<String, Node>,
}
impl JsonResolver {
    pub fn new(nodes: Vec<Node>) -> Self {
        let nodes = nodes
            .into_iter()
            .map(|node| (node.id().to_string(), node))
            .collect::<HashMap<_, _>>();
        Self { nodes }
    }
}

impl QueryResolver for JsonResolver {
    fn resolve_node_by_id(ctx: ResolverContext) -> FieldFuture {
        FieldFuture::new(async move {
            let resolver = ctx.data_unchecked::<JsonResolver>();
            let id = ctx.args.get("id").unwrap();
            let id = id.string().unwrap();
            let node = resolver.nodes.get(id);
            if let Some(node) = node {
                Ok(Some(node.to_typed_field_value()))
            } else {
                Ok(None)
            }
        })
    }
    fn resolve_node_by_ids(ctx: ResolverContext) -> FieldFuture {
        FieldFuture::new(async move {
            let resolver = ctx.data_unchecked::<JsonResolver>();
            let ids = ctx.args.get("ids").unwrap();
            let ids = ids.list().unwrap();
            let ids = ids
                .iter()
                .map(|id| id.string().unwrap().to_string())
                .collect::<Vec<_>>();
            let nodes = ids
                .iter()
                .map(|id| {
                    let node = resolver.nodes.get(id).unwrap();
                    node.to_typed_field_value()
                })
                .collect::<Vec<_>>();
            Ok(Some(FieldValue::list(nodes)))
        })
    }
}

impl NodeResolver for JsonResolver {
    fn resolve_id(
        ctx: async_graphql::dynamic::ResolverContext,
    ) -> async_graphql::dynamic::FieldFuture {
        FieldFuture::new(async move {
            let parent = ctx.parent_value.try_downcast_ref::<Node>()?;
            Ok(Some(FieldValue::value(parent.id())))
        })
    }
    fn resolve_data(ctx: ResolverContext) -> FieldFuture {
        FieldFuture::new(async move {
            let parent = ctx.parent_value.try_downcast_ref::<Node>()?;
            let path_node = ctx.path_node.unwrap();
            let field_name = path_node.field_name();
            let value = parent.data()[field_name].as_str().unwrap();
            Ok(Some(FieldValue::value(value)))
        })
    }
    fn resolve_ref(ctx: ResolverContext) -> FieldFuture {
        FieldFuture::new(async move {
            let parent = ctx.parent_value.try_downcast_ref::<Node>()?;
            let path_node = ctx.path_node.unwrap();
            let field_name = path_node.field_name();
            let id = parent.data()[field_name].as_str().unwrap();
            let resolver = ctx.data_unchecked::<JsonResolver>();
            let node = resolver.nodes.get(id).unwrap();
            Ok(Some(node.to_field_value()))
        })
    }
    fn resolve_connection(_ctx: ResolverContext) -> FieldFuture {
        unimplemented!()
    }
}

impl ConnectionResolver for JsonResolver {
    fn resolve_total_count(_ctx: ResolverContext) -> FieldFuture {
        unimplemented!()
    }
    fn resolve_nodes(_ctx: ResolverContext) -> FieldFuture {
        unimplemented!()
    }
    fn resolve_edges(_ctx: ResolverContext) -> FieldFuture {
        unimplemented!()
    }
    fn resolve_page_info(_ctx: ResolverContext) -> FieldFuture {
        unimplemented!()
    }
}

impl EdgeResolver for JsonResolver {
    fn resolve_node(_ctx: ResolverContext) -> FieldFuture {
        unimplemented!()
    }
    fn resolve_cursor(_ctx: ResolverContext) -> FieldFuture {
        unimplemented!()
    }
}

impl PageInfoResolver for JsonResolver {
    fn resolve_has_next_page(
        ctx: async_graphql::dynamic::ResolverContext,
    ) -> async_graphql::dynamic::FieldFuture {
        FieldFuture::new(async move {
            let parent = ctx.parent_value.try_downcast_ref::<JsonValue>()?;
            Ok(Some(FieldValue::value(
                parent["has_next_page"].as_bool().unwrap(),
            )))
        })
    }
    fn resolve_has_previous_page(ctx: ResolverContext) -> FieldFuture {
        FieldFuture::new(async move {
            let parent = ctx.parent_value.try_downcast_ref::<JsonValue>()?;
            Ok(Some(FieldValue::value(
                parent["has_previous_page"].as_bool().unwrap(),
            )))
        })
    }
    fn resolve_start_cursor(ctx: ResolverContext) -> FieldFuture {
        FieldFuture::new(async move {
            let parent = ctx.parent_value.try_downcast_ref::<JsonValue>()?;
            Ok(Some(FieldValue::value(
                parent["start_cursor"].as_str().unwrap(),
            )))
        })
    }
    fn resolve_end_cursor(ctx: ResolverContext) -> FieldFuture {
        FieldFuture::new(async move {
            let parent = ctx.parent_value.try_downcast_ref::<JsonValue>()?;
            Ok(Some(FieldValue::value(
                parent["end_cursor"].as_str().unwrap(),
            )))
        })
    }
}

use super::edge::*;
use super::node::*;
use super::prelude::*;

#[async_trait]
pub trait DynamicLoader: Send + Sync + 'static {
    async fn load_node_by_id(
        &self,
        id: &DynamicNodeId,
    ) -> Result<Option<DynamicNode>, ()>;
    async fn load_nodes_by_id(
        &self,
        ids: &[DynamicNodeId],
    ) -> Result<Vec<Option<DynamicNode>>, ()>;
    async fn load_edges(
        &self,
        type_id: &DynamicEdgeTypeId,
        tail_local_id: &DynamicNodeLocalId,
    ) -> Result<Vec<DynamicEdge>, ()>;
}

pub struct TestLoader {
    pub nodes: HashMap<DynamicNodeId, DynamicNode>,
    pub edges: HashMap<(DynamicEdgeTypeId, DynamicNodeLocalId), DynamicEdge>,
}
impl TestLoader {
    pub fn new(nodes: Vec<DynamicNode>, edges: Vec<DynamicEdge>) -> Self {
        let nodes = nodes
            .into_iter()
            .map(|node| (node.id(), node))
            .collect::<HashMap<_, _>>();
        let edges = edges
            .into_iter()
            .map(|edge| ((edge.type_id().clone(), edge.tail_local_id().clone()), edge))
            .collect::<HashMap<_, _>>();
        Self { nodes, edges }
    }
}

#[async_trait]
impl DynamicLoader for TestLoader {
    async fn load_node_by_id(
        &self,
        id: &DynamicNodeId,
    ) -> Result<Option<DynamicNode>, ()> {
        let node = self.nodes.get(id);
        Ok(node.cloned())
    }

    async fn load_nodes_by_id(
        &self,
        ids: &[DynamicNodeId],
    ) -> Result<Vec<Option<DynamicNode>>, ()> {
        let mut nodes = Vec::new();
        for id in ids {
            let node = self.load_node_by_id(id).await?;
            nodes.push(node);
        }
        Ok(nodes)
    }

    async fn load_edges(
        &self,
        type_id: &DynamicEdgeTypeId,
        tail_local_id: &DynamicNodeLocalId,
    ) -> Result<Vec<DynamicEdge>, ()> {
        let edges = self
            .edges
            .iter()
            .filter(|(id, _edge)| id.0 == *type_id && id.1 == *tail_local_id)
            .map(|(_, edge)| edge)
            .cloned()
            .collect();
        Ok(edges)
    }
}

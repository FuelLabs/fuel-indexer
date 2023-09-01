use super::edge::*;
use super::node::*;
use super::self_prelude::*;

pub struct DynLoader {
    pub store: Arc<Mutex<dyn store::Store>>,
}

pub type DynLoaderResult<T> = anyhow::Result<T, DynLoaderError>;

#[derive(thiserror::Error, Debug)]
pub enum DynLoaderError {
    #[error(transparent)]
    DataStore(#[from] store::StoreError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl DynLoader {
    pub fn new(store: Arc<Mutex<dyn store::Store>>) -> Self {
        Self { store }
    }

    pub async fn load_node_one(
        &self,
        id: &DynNodeId,
    ) -> DynLoaderResult<Option<DynNode>> {
        let store = self.store.lock().await;
        let object = store.obj_get(id).await?;
        Ok(object.map(|object| DynNode::new(id.clone(), object)))
    }
    pub async fn load_node_many(
        &self,
        ids: &[DynNodeId],
    ) -> DynLoaderResult<Vec<Option<DynNode>>> {
        // let ids = ids.iter().map(|id| id.to_string()).collect::<Vec<_>>();
        let store = self.store.lock().await;
        let objects = store
            .obj_get_many(&ids.iter().map(|id| id.to_string()).collect::<Vec<_>>())
            .await?;
        Ok(objects
            .into_iter()
            .enumerate()
            .map(|(i, object)| object.map(|object| DynNode::new(ids[i].clone(), object)))
            .collect())
    }
    pub async fn load_node_edge_count(
        &self,
        id: &DynNodeId,
        edge_type_id: &DynEdgeTypeId,
    ) -> DynLoaderResult<u64>
    where
        Self: 'static,
    {
        let store = self.store.lock().await;
        let key = store::AssocKey(id.to_string(), edge_type_id.to_string());
        let count = store.assoc_count(&key).await?;
        Ok(count)
    }
    pub async fn load_node_edges(
        &self,
        id: &DynNodeId,
        edge_type_id: &DynEdgeTypeId,
    ) -> DynLoaderResult<Vec<DynEdge>> {
        let store = self.store.lock().await;
        let key = store::AssocKey(id.to_string(), edge_type_id.to_string());
        let assocs = store.assoc_range(&key, 0, 9999).await?;
        Ok(assocs
            .into_iter()
            .map(|assoc| DynEdge::new(edge_type_id, assoc.id().clone(), assoc))
            .collect())
    }
}

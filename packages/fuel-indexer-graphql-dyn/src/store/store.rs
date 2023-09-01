use super::assoc::*;
use super::obj::*;
use super::self_prelude::*;
use super::store_type::*;

type Result<T> = anyhow::Result<T, StoreError>;

/// See: https://research.facebook.com/publications/tao-facebooks-distributed-data-store-for-the-social-graph/
#[async_trait]
pub trait Store: Send + Sync {
    /// Returns the type of the store.
    fn r#type(&self) -> &StoreType;
    //
    /// Gets a single object by its ID.
    async fn obj_get(&self, id: &ObjId) -> Result<Option<Obj>>;
    /// Gets objects by their IDs.
    async fn obj_get_many(&self, ids: &[ObjId]) -> Result<Vec<Option<Obj>>>;
    // Gets associations of an object by their head IDs.
    async fn assoc_get(
        &self,
        key: &AssocKey,
        ids: &[ObjId],
        high: Option<AssocTime>,
        low: Option<AssocTime>,
    ) -> Result<Vec<Option<Assoc>>>;
    // Gets the count of associations an object has.
    async fn assoc_count(&self, key: &AssocKey) -> Result<u64>;
    // Gets associations of an object.
    async fn assoc_range(
        &self,
        key: &AssocKey,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Assoc>>;
    // Gets associations of an object in the given time range.
    async fn assoc_time_range(
        &self,
        key: &AssocKey,
        high: AssocTime,
        low: AssocTime,
        limit: u64,
    ) -> Result<Vec<Assoc>>;
}

pub type StoreResult<T> = anyhow::Result<T, StoreError>;

#[derive(thiserror::Error, Debug)]
pub enum StoreError {
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

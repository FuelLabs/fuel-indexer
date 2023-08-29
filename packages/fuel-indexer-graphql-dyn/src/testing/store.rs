use super::self_prelude::*;
use super::store_type::*;
use crate::store::*;

#[derive(Clone, Debug)]
pub struct TestStore {
    pub r#type: StoreType,
    pub obj_map: IndexMap<ObjId, Obj>,
    pub assoc_map: IndexMap<AssocKey, Vec<Assoc>>,
    pub time: AssocTime,
}

pub fn new_test_store() -> Result<TestStore, ()> {
    let store_type = new_test_store_type().unwrap();
    let mut store = TestStore::new(store_type);
    for (chain_idx, chain_id) in (0..10).map(|i| format!("Chain:{}", i)).enumerate() {
        store.obj_add(
            "Chain",
            &chain_id,
            json!({
                "display_name": format!("Chain #{}", chain_idx),
            }),
        );
        for (block_idx, block_id) in (0..10)
            .map(|i| format!("Block:{}-{}", chain_idx, i))
            .enumerate()
        {
            store.obj_add(
                "Block",
                &block_id,
                json!({
                    "number": block_idx,
                    "hash": "0x00",
                    "parent_hash": "0x00",
                    "chain_id": &chain_id,
                }),
            );
            store.assoc_add("Chain", "Has", "Block", &chain_id, &block_id, json!(null));
            for (transaction_idx, transaction_id) in (0..10)
                .map(|i| format!("Transaction:{}-{}-{}", chain_idx, block_idx, i))
                .enumerate()
            {
                store.obj_add(
                    "Transaction",
                    &transaction_id,
                    json!({
                        "index": transaction_idx,
                        "hash": "0x00",
                        "block_hash": "0",
                        "gas_price": 12312,
                        "gas_limit": 32423,
                        "chain_id": &chain_id,
                    }),
                );
                store.assoc_add(
                    "Block",
                    "Has",
                    "Transaction",
                    &block_id,
                    &transaction_id,
                    json!(null),
                );
                store.assoc_add(
                    "Chain",
                    "Has",
                    "Transaction",
                    &chain_id,
                    &transaction_id,
                    json!(null),
                );
            }
        }
    }
    Ok(store)
}

impl TestStore {
    pub fn new(r#type: StoreType) -> Self {
        Self {
            r#type,
            obj_map: IndexMap::new(),
            assoc_map: IndexMap::new(),
            time: 0,
        }
    }

    pub fn obj_add(
        &mut self,
        type_id: impl Into<ObjTypeId>,
        id: impl Into<ObjId>,
        data: impl Into<Data>,
    ) {
        let id = id.into();
        let type_id = type_id.into();
        let data = data.into();

        let object = Obj(type_id, data);
        self.obj_map.insert(id, object);
    }
    pub fn assoc_add(
        &mut self,
        tail_type_id: impl Into<ObjTypeId>,
        verb: impl Into<String>,
        head_type_id: impl Into<ObjTypeId>,
        tail_id: impl Into<ObjId>,
        head_id: impl Into<ObjId>,
        data: impl Into<Data>,
    ) {
        let tail_type_id = tail_type_id.into();
        let head_type_id = head_type_id.into();
        let type_id = format!(
            "{}{}{}",
            tail_type_id.clone(),
            verb.into(),
            head_type_id.clone()
        );
        let tail_id = tail_id.into();
        let head_id = head_id.into();
        let data = data.into();

        let key = AssocKey(tail_id, type_id);
        let assoc = Assoc(head_id, self.time, data);
        self.time += 1;
        self.assoc_map
            .entry(key)
            .or_insert_with(Vec::new)
            .push(assoc);
    }
}

#[async_trait]
impl Store for TestStore {
    fn r#type(&self) -> &StoreType {
        &self.r#type
    }

    async fn obj_get(&self, id: &ObjId) -> StoreResult<Option<Obj>> {
        Ok(self.obj_map.get(id).cloned())
    }
    async fn obj_get_many(&self, ids: &[ObjId]) -> StoreResult<Vec<Option<Obj>>> {
        Ok(ids.iter().map(|id| self.obj_map.get(id).cloned()).collect())
    }
    async fn assoc_get(
        &self,
        _key: &AssocKey,
        _ids: &[ObjId],
        _high: Option<AssocTime>,
        _low: Option<AssocTime>,
    ) -> StoreResult<Vec<Option<Assoc>>> {
        todo!()
    }
    async fn assoc_count(&self, _key: &AssocKey) -> StoreResult<u64> {
        todo!()
    }
    async fn assoc_range(
        &self,
        key: &AssocKey,
        offset: u64,
        limit: u64,
    ) -> StoreResult<Vec<Assoc>> {
        Ok(self
            .assoc_map
            .get(key)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect())
    }
    async fn assoc_time_range(
        &self,
        _key: &AssocKey,
        _high: AssocTime,
        _low: AssocTime,
        _limit: u64,
    ) -> StoreResult<Vec<Assoc>> {
        todo!()
    }
}

#[test]
fn test_store() {
    use insta::*;

    let store = new_test_store().unwrap();
    assert_debug_snapshot!(store);
}

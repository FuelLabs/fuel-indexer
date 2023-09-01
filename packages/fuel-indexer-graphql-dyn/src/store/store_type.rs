use super::assoc_type::*;
use super::data_type::*;
use super::obj_type::*;
use super::self_prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreType {
    pub data: IndexMap<DataTypeId, DataType>,
    pub obj: IndexMap<ObjTypeId, ObjType>,
    pub assoc: IndexMap<AssocTypeId, AssocType>,
}

impl StoreType {
    pub fn data_type(&self, id: &DataTypeId) -> &DataType {
        self.data.get(id).unwrap()
    }
    pub fn obj_type(&self, id: &ObjTypeId) -> &ObjType {
        self.obj.get(id).unwrap()
    }
    pub fn assoc_type(&self, id: &AssocTypeId) -> &AssocType {
        self.assoc.get(id).unwrap()
    }
}

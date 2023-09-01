use super::data::*;
use super::obj_type::*;
use super::self_prelude::*;

pub type ObjId = String;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Obj(pub ObjTypeId, pub Data);

impl Obj {
    pub fn new(obj_type_id: ObjTypeId, data: Data) -> Self {
        Self(obj_type_id, data)
    }

    pub fn type_id(&self) -> &ObjTypeId {
        &self.0
    }
    pub fn data(&self) -> &Data {
        &self.1
    }
}

use super::assoc_type::*;
use super::data::*;
use super::obj::*;
use super::self_prelude::*;

pub type AssocTime = u64;

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct AssocKey(pub ObjId, pub AssocTypeId);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Assoc(pub ObjId, pub AssocTime, pub Data);

impl Assoc {
    pub fn id(&self) -> &ObjId {
        &self.0
    }
    pub fn time(&self) -> &AssocTime {
        &self.1
    }
    pub fn data(&self) -> &Data {
        &self.2
    }
}

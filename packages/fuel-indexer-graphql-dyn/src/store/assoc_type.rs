use super::data_type::*;
use super::meta::*;
use super::obj_type::*;
use super::self_prelude::*;

pub type AssocTypeId = String;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssocType {
    pub name: Name,
    pub tail: ObjTypeId,
    pub head: ObjTypeId,
    pub fields: IndexMap<DataFieldId, DataFieldType>,
}

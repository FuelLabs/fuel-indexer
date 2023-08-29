use super::data_type::*;
use super::meta::*;
use super::self_prelude::*;

pub type ObjTypeId = String;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjType {
    pub name: Name,
    pub fields: IndexMap<DataFieldId, DataFieldType>,
}

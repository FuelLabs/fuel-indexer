use super::prelude::*;

#[derive(Clone, Hash, PartialEq, Eq, Debug, EnumString, strum::Display)]
pub enum DynamicDataType {
    Int,
    String,
    // TODO: Implement remaining types
}

pub type DynamicData = serde_json::Value;

pub type DynamicFieldId = String;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct DynamicDataField(DynamicFieldId, DynamicDataType);
impl DynamicDataField {
    pub fn new(name: impl Into<String>, data_type: impl Into<DynamicDataType>) -> Self {
        Self(name.into(), data_type.into())
    }

    pub fn id(&self) -> &DynamicFieldId {
        &self.0
    }
    pub fn data_type(&self) -> &DynamicDataType {
        &self.1
    }
}

#[derive(Clone)]
pub struct DynamicDataFieldResolver {
    pub data_field: DynamicDataField,
}
impl DynamicDataFieldResolver {
    pub fn new(data_field: DynamicDataField) -> Self {
        Self { data_field }
    }

    pub fn resolve(self, data: &DynamicData) -> Option<FieldValue> {
        let value = data.get(self.data_field.id()).unwrap();
        match self.data_field.data_type() {
            DynamicDataType::String => {
                let value: String = value.as_str().unwrap().to_string();
                Some(FieldValue::value(value))
            }
            DynamicDataType::Int => {
                let value: i32 = value.as_i64().unwrap() as i32;
                Some(FieldValue::value(value))
            }
        }
    }
}

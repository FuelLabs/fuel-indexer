use super::assoc_type::*;
use super::data_type::*;
use super::meta::*;
use super::obj_type::*;
use super::self_prelude::*;
use super::store_type::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StoreTypeBuilder {
    pub data: IndexMap<DataTypeId, DataTypeBuilder>,
    pub obj: IndexMap<ObjTypeId, ObjTypeBuilder>,
    pub assoc: IndexMap<AssocTypeId, AssocTypeBuilder>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DataTypeBuilder {
    Composite(StructTypeBuilder),
    Enum(EnumTypeBuilder),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StructTypeBuilder {
    pub name: Name,
    pub fields: IndexMap<DataFieldId, DataFieldType>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnumTypeBuilder {
    pub name: Name,
    pub variants: IndexMap<DataFieldId, DataFieldType>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjTypeBuilder {
    pub name: Name,
    pub fields: IndexMap<DataFieldId, DataFieldType>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssocTypeBuilder {
    pub name: Name,
    pub tail: ObjTypeId,
    pub head: ObjTypeId,
    pub fields: IndexMap<DataFieldId, DataFieldType>,
}

impl StoreTypeBuilder {
    pub fn data(
        &mut self,
        name: impl Into<String>,
    ) -> (DataTypeId, &mut DataTypeBuilder) {
        let name = Name::new_pascal(name);
        self.data
            .iter_mut()
            .find(|(_, data)| data.name() == name)
            .map(|(id, data)| (id.clone(), data))
            .unwrap_or_else(|| panic!("Data not found: {}", name))
    }
    pub fn obj(&mut self, name: impl Into<String>) -> (ObjTypeId, &mut ObjTypeBuilder) {
        let name = Name::new_pascal(name);
        self.obj
            .values_mut()
            .find(|obj| obj.name == name)
            .map(|obj| (obj.name.to_snake_string(), obj))
            .unwrap_or_else(|| panic!("Obj not found: {}", name))
    }
    pub fn assoc(
        &mut self,
        tail: &ObjTypeId,
        verb: impl Into<String>,
        head: &ObjTypeId,
    ) -> (AssocTypeId, &mut AssocTypeBuilder) {
        let verb = verb.into();
        let name = format!("{}{}{}", tail, verb, head);
        let name = Name::new_pascal(name);
        self.assoc
            .values_mut()
            .find(|assoc| assoc.name == name)
            .map(|assoc| (assoc.name.to_snake_string(), assoc))
            .unwrap_or_else(|| panic!("Assoc not found: {}", name))
    }

    pub fn define_enum(
        &mut self,
        name: impl Into<String>,
    ) -> (DataTypeId, &mut EnumTypeBuilder) {
        let name = Name::new_pascal(name);
        let id = DataTypeId::Name(name.clone());

        let builder = self
            .data
            .entry(id.clone())
            .or_insert_with(|| DataTypeBuilder::Enum(EnumTypeBuilder::new(name)));
        match builder {
            DataTypeBuilder::Enum(builder) => (id, builder),
            _ => panic!("Expected enum type builder"),
        }
    }
    pub fn define_composite(
        &mut self,
        name: impl Into<String>,
    ) -> (DataTypeId, &mut StructTypeBuilder) {
        let name = Name::new_pascal(name);
        let id = DataTypeId::Name(name.clone());

        let builder = self
            .data
            .entry(id.clone())
            .or_insert_with(|| DataTypeBuilder::Composite(StructTypeBuilder::new(name)));
        match builder {
            DataTypeBuilder::Composite(builder) => (id, builder),
            _ => panic!("Expected enum type builder"),
        }
    }
    pub fn define_obj(
        &mut self,
        name: impl Into<String>,
    ) -> (ObjTypeId, &mut ObjTypeBuilder) {
        let name = name.into();
        let id = name.clone();
        (
            id.clone(),
            self.obj
                .entry(id)
                .or_insert_with(|| ObjTypeBuilder::new(name)),
        )
    }
    pub fn define_assoc(
        &mut self,
        tail: &ObjTypeId,
        verb: impl Into<String>,
        head: &ObjTypeId,
    ) -> (ObjTypeId, &mut AssocTypeBuilder) {
        let verb = verb.into();
        let name = Name::new_pascal(format!("{}{}{}", tail, verb, head));
        let id = name.to_pascal_string();
        (
            id.clone(),
            self.assoc
                .entry(id)
                .or_insert_with(|| AssocTypeBuilder::new(&name, tail, head)),
        )
    }

    pub fn finish(self) -> Result<StoreType, ()> {
        let mut data_types: IndexMap<DataTypeId, DataType> = Default::default();
        let mut obj_types: IndexMap<ObjTypeId, ObjType> = Default::default();
        let mut assoc_types: IndexMap<AssocTypeId, AssocType> = Default::default();

        {
            use DataType::*;
            data_types.insert(DataTypeId::Unit, Unit);
            data_types.insert(DataTypeId::Bool, Bool);
            data_types.insert(DataTypeId::U8, U8);
            data_types.insert(DataTypeId::U16, U16);
            data_types.insert(DataTypeId::U32, U32);
            data_types.insert(DataTypeId::U64, U64);
            data_types.insert(DataTypeId::B256, B256);
            data_types.insert(DataTypeId::Byte, Byte);
            data_types.insert(DataTypeId::Bytes, Bytes);
            data_types.insert(DataTypeId::String, String);
        }

        for (id, builder) in self.data {
            data_types.insert(id, builder.finish().unwrap());
        }
        for (id, builder) in self.obj {
            let name = builder.name;
            let mut fields: IndexMap<DataFieldId, DataFieldType> = Default::default();

            for (field_name, type_id) in builder.fields {
                fields.insert(field_name.clone(), type_id.clone());
            }

            obj_types.insert(id, ObjType { name, fields });
        }
        for (id, builder) in self.assoc {
            let name = builder.name;
            let tail = builder.tail;
            let head = builder.head;
            let mut fields: IndexMap<DataFieldId, DataFieldType> = Default::default();

            for (field_name, type_id) in builder.fields {
                fields.insert(field_name.clone(), type_id.clone());
            }

            assoc_types.insert(
                id,
                AssocType {
                    name,
                    tail,
                    head,
                    fields,
                },
            );
        }

        Ok(StoreType {
            data: data_types,
            obj: obj_types,
            assoc: assoc_types,
        })
    }
}

impl DataTypeBuilder {
    pub fn name(&self) -> Name {
        match self {
            Self::Composite(builder) => builder.name.clone(),
            Self::Enum(builder) => builder.name.clone(),
        }
    }
}

impl StructTypeBuilder {
    pub fn new(name: Name) -> Self {
        Self {
            name,
            fields: Default::default(),
        }
    }

    pub fn define_field(&mut self, name: impl Into<String>, type_id: &DataTypeId) {
        let name = Name::new_snake(name);
        let id = name.to_snake_string();
        self.fields.insert(
            id,
            DataFieldType {
                name,
                type_id: type_id.clone(),
            },
        );
    }
}

impl EnumTypeBuilder {
    pub fn new(name: Name) -> Self {
        Self {
            name,
            variants: Default::default(),
        }
    }

    pub fn define_variant(&mut self, name: impl Into<String>, type_id: &DataTypeId) {
        let name = Name::new_pascal(name);
        let id = name.to_snake_string();
        self.variants.insert(
            id,
            DataFieldType {
                name,
                type_id: type_id.clone(),
            },
        );
    }
}

impl DataTypeBuilder {
    pub fn finish(self) -> Result<DataType, ()> {
        match self {
            Self::Composite(builder) => {
                let mut fields: IndexMap<String, DataFieldType> = Default::default();
                for (field_name, field_type) in builder.fields {
                    fields.insert(field_name, field_type);
                }
                Ok(DataType::Composite(builder.name, fields))
            }
            Self::Enum(builder) => {
                let mut variants: IndexMap<String, DataFieldType> = Default::default();
                for (variant_name, variant_type) in builder.variants {
                    variants.insert(variant_name, variant_type);
                }
                Ok(DataType::Enum(builder.name, variants))
            }
        }
    }
}

impl ObjTypeBuilder {
    pub fn new(name: String) -> Self {
        let name = Name::new_pascal(name);
        Self {
            name,
            fields: Default::default(),
        }
    }

    pub fn define_field(&mut self, name: impl Into<String>, type_id: &DataTypeId) {
        let name = Name::new_snake(name);
        let id = name.to_snake_string();
        self.fields.insert(
            id,
            DataFieldType {
                name,
                type_id: type_id.clone(),
            },
        );
    }
}

impl AssocTypeBuilder {
    pub fn new(name: &Name, tail: &ObjTypeId, head: &ObjTypeId) -> Self {
        let name = name.clone();
        let tail = tail.clone();
        let head = head.clone();
        Self {
            name,
            tail,
            head,
            fields: Default::default(),
        }
    }

    pub fn define_field(&mut self, name: impl Into<String>, type_id: &DataTypeId) {
        let name = Name::new_snake(name);
        let id = name.to_snake_string();
        self.fields.insert(
            id,
            DataFieldType {
                name,
                type_id: type_id.clone(),
            },
        );
    }
}

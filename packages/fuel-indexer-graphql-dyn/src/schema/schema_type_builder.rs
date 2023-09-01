use super::data::*;
use super::edge::*;
use super::node::*;
use super::schema_type::*;
use super::self_prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DynSchemaTypeBuilder {
    pub store: store::StoreType,
    pub data: IndexMap<DynDataTypeId, DynDataTypeBuilder>,
    pub node: IndexMap<DynNodeTypeId, DynNodeTypeBuilder>,
    pub edge: IndexMap<DynEdgeTypeId, DynEdgeTypeBuilder>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DynDataTypeBuilder {
    Object(DynObjectTypeBuilder),
    Enum(DynEnumTypeBuilder),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DynObjectTypeBuilder {
    pub name: Name,
    pub fields: IndexMap<DynDataFieldId, DynDataFieldType>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DynEnumTypeBuilder {
    pub name: Name,
    pub variants: IndexMap<DynDataFieldId, DynDataFieldType>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DynNodeTypeBuilder {
    pub name: Name,
    pub fields: IndexMap<DynDataFieldId, DynNodeFieldType>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DynNodeFieldTypeBuilder {
    Data(DynDataFieldType),
    Ref(DynNodeRefFieldType),
    Connection(DynNodeConnectionFieldType),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DynEdgeTypeBuilder {
    pub name: Name,
    pub tail: DynNodeTypeId,
    pub head: DynNodeTypeId,
    pub fields: IndexMap<DynDataFieldId, DynDataFieldType>,
}

impl DynSchemaTypeBuilder {
    pub fn new(store_type: &store::StoreType) -> Self {
        Self {
            store: store_type.clone(),
            data: Default::default(),
            node: Default::default(),
            edge: Default::default(),
        }
        .define_store_types()
    }

    pub(super) fn define_store_types(mut self) -> Self {
        let store = self.store.clone();
        for (_store_id, data) in &store.data {
            use store::DataType::*;
            match data {
                Unit | Bool | U8 | U16 | U32 | U64 | B256 | Byte | Bytes | String => {}
                Array(_id) => {
                    todo!()
                }
                Composite(name, fields) => {
                    let builder = self.define_object(name.to_pascal_string());
                    for (field_id, field) in fields {
                        let field_type_id: DynDataTypeId = field.type_id.clone().into();
                        builder.fields.insert(
                            field_id.clone(),
                            DynDataFieldType {
                                name: field.name.clone(),
                                data_type_id: field_type_id,
                                store_data_field_id: field_id.clone(),
                            },
                        );
                    }
                }
                Enum(name, variants) => {
                    let builder = self.define_enum(name.to_pascal_string());
                    for (field_id, field) in variants {
                        let field_type_id: DynDataTypeId = field.type_id.clone().into();
                        builder.variants.insert(
                            field_id.clone(),
                            DynDataFieldType {
                                name: field.name.clone(),
                                data_type_id: field_type_id,
                                store_data_field_id: field_id.clone(),
                            },
                        );
                    }
                }
            }
        }
        for (id, obj) in &store.obj {
            let (_, node) = self.define_node(id);
            for (field_id, field) in &obj.fields {
                let field_type_id: DynDataTypeId = field.type_id.clone().into();
                node.define_data(field.name.clone(), field_id, field_type_id)
            }
        }
        for (id, assoc) in &store.assoc {
            let tail = assoc.tail.clone();
            let tail: DynNodeTypeId = tail.into();
            let head = assoc.head.clone();
            let head: DynNodeTypeId = head.into();
            let (_, edge) = self.define_edge(id, tail, head);
            for (field_id, field) in &assoc.fields {
                let field_type_id: DynDataTypeId = field.type_id.clone().into();
                edge.define_data(field.name.clone(), field_id, field_type_id)
            }
        }
        self
    }
    pub fn define_enum(&mut self, name: impl Into<String>) -> &mut DynEnumTypeBuilder {
        let name = Name::new_pascal(name);
        let id = DynDataTypeId::Name(name.clone());
        match self
            .data
            .entry(id)
            .or_insert(DynDataTypeBuilder::Enum(DynEnumTypeBuilder::new(name)))
        {
            DynDataTypeBuilder::Enum(builder) => builder,
            _ => unreachable!(),
        }
    }
    pub fn define_object(
        &mut self,
        name: impl Into<String>,
    ) -> &mut DynObjectTypeBuilder {
        let name = Name::new_pascal(name);
        let id = DynDataTypeId::Name(name.clone());
        match self
            .data
            .entry(id)
            .or_insert(DynDataTypeBuilder::Object(DynObjectTypeBuilder::new(name)))
        {
            DynDataTypeBuilder::Object(builder) => builder,
            _ => unreachable!(),
        }
    }
    pub fn define_node(
        &mut self,
        name: impl Into<String>,
    ) -> (DynNodeTypeId, &mut DynNodeTypeBuilder) {
        let name = Name::new_pascal(name);
        let id: DynNodeTypeId = name.to_pascal_string().into();
        (
            id.clone(),
            self.node
                .entry(id)
                .or_insert_with(|| DynNodeTypeBuilder::new(name)),
        )
    }
    pub fn define_edge(
        &mut self,
        name: impl Into<String>,
        tail_type_id: impl Into<DynNodeTypeId>,
        head_type_id: impl Into<DynNodeTypeId>,
    ) -> (DynEdgeTypeId, &mut DynEdgeTypeBuilder) {
        let name = Name::new_pascal(name);
        let id: DynEdgeTypeId = name.to_pascal_string();
        (
            id.clone(),
            self.edge.entry(id).or_insert_with(|| {
                DynEdgeTypeBuilder::new(name, tail_type_id, head_type_id)
            }),
        )
    }

    pub fn finish(self) -> Result<DynSchemaType, ()> {
        let mut data_types: IndexMap<DynDataTypeId, DynDataType> = Default::default();
        let mut node_types: IndexMap<DynNodeTypeId, DynNodeType> = Default::default();
        let mut edge_types: IndexMap<DynEdgeTypeId, DynEdgeType> = Default::default();

        {
            use DynDataType::*;
            data_types.insert(DynDataTypeId::Unit, Unit);
            data_types.insert(DynDataTypeId::Boolean, Boolean);
            data_types.insert(DynDataTypeId::U8, U8);
            data_types.insert(DynDataTypeId::U16, U16);
            data_types.insert(DynDataTypeId::U32, U32);
            data_types.insert(DynDataTypeId::U64, U64);
            data_types.insert(DynDataTypeId::B256, B256);
            data_types.insert(DynDataTypeId::Bytes, Bytes);
            data_types.insert(DynDataTypeId::String, String);
        }

        for (_id, data) in self.data {
            let id = data.id();
            data_types.insert(
                id,
                match data {
                    DynDataTypeBuilder::Object(builder) => builder.finish().unwrap(),
                    DynDataTypeBuilder::Enum(builder) => builder.finish().unwrap(),
                },
            );
        }
        for (id, builder) in self.node {
            let name = builder.name;
            let mut fields: IndexMap<DynDataFieldId, DynNodeFieldType> =
                Default::default();

            for (field_id, field) in builder.fields {
                match field {
                    DynNodeFieldType::Data(data_field) => {
                        fields.insert(
                            field_id.clone(),
                            DynNodeFieldType::Data(DynDataFieldType {
                                name: data_field.name.clone(),
                                data_type_id: data_field.data_type_id().clone(),
                                store_data_field_id: data_field
                                    .store_data_field_id
                                    .clone(),
                            }),
                        );
                    }
                    DynNodeFieldType::Ref(ref_field) => {
                        fields.insert(
                            field_id.clone(),
                            DynNodeFieldType::Ref(DynNodeRefFieldType {
                                name: ref_field.name.clone(),
                                store_id: ref_field.store_id.clone(),
                                ref_node_type_id: ref_field.ref_node_type_id.clone(),
                            }),
                        );
                    }
                    DynNodeFieldType::Connection(connection_field) => {
                        fields.insert(
                            field_id,
                            DynNodeFieldType::Connection(DynNodeConnectionFieldType {
                                name: connection_field.name.clone(),
                                edge_type_id: connection_field.edge_type_id.clone(),
                            }),
                        );
                    }
                }
            }

            node_types.insert(id, DynNodeType { name, fields });
        }
        for (id, builder) in self.edge {
            let name = builder.name;
            let tail_type_id = builder.tail.clone();
            let head_type_id = builder.head.clone();

            let mut fields: IndexMap<DynDataFieldId, DynDataFieldType> =
                Default::default();
            for (field_name, field_type) in builder.fields {
                let field_id = field_name.clone();
                fields.insert(
                    field_id,
                    DynDataFieldType {
                        name: field_type.name.clone(),
                        data_type_id: field_type.data_type_id().clone(),
                        store_data_field_id: field_type.store_data_field_id.clone(),
                    },
                );
            }

            edge_types.insert(
                id,
                DynEdgeType {
                    name,
                    tail: tail_type_id,
                    head: head_type_id,
                    fields,
                },
            );
        }

        Ok(DynSchemaType {
            data: data_types,
            node: node_types,
            edge: edge_types,
        })
    }
}

impl DynObjectTypeBuilder {
    pub fn new(name: Name) -> Self {
        Self {
            name,
            fields: Default::default(),
        }
    }

    pub fn finish(self) -> Result<DynDataType, ()> {
        let name = self.name;
        let fields = self
            .fields
            .into_iter()
            .map(|(id, field)| (id, field))
            .collect();
        Ok(DynDataType::Object(name, fields))
    }
}

impl DynEnumTypeBuilder {
    pub fn new(name: Name) -> Self {
        Self {
            name,
            variants: Default::default(),
        }
    }

    pub fn finish(self) -> Result<DynDataType, ()> {
        let name = self.name;
        let variants = self
            .variants
            .into_iter()
            .map(|(id, field)| (id, field))
            .collect();
        Ok(DynDataType::Enum(name, variants))
    }
}

impl DynDataTypeBuilder {
    pub fn id(&self) -> DynDataTypeId {
        match self {
            Self::Object(DynObjectTypeBuilder { name, fields: _ }) => {
                DynDataTypeId::Name(Name::new_pascal(format!("{}", name)))
            }
            Self::Enum(DynEnumTypeBuilder { name, variants: _ }) => {
                DynDataTypeId::Name(Name::new_pascal(format!("{}", name)))
            }
        }
    }
}

impl DynNodeTypeBuilder {
    pub fn new(name: Name) -> Self {
        Self {
            name,
            fields: Default::default(),
        }
    }

    pub fn define_data(
        &mut self,
        name: Name,
        data_field_id: impl Into<DynDataFieldId>,
        data_type_id: impl Into<DynDataTypeId>,
    ) {
        let id = name.to_camel_string();
        let field = DynDataFieldType {
            name,
            data_type_id: data_type_id.into(),
            store_data_field_id: data_field_id.into(),
        };
        self.fields.insert(id, DynNodeFieldType::Data(field));
    }
    pub fn define_ref(
        &mut self,
        name: impl Into<String>,
        data_field_id: impl Into<DynDataFieldId>,
        node_type_id: &DynNodeTypeId,
    ) {
        let name = Name::new_camel(name);
        let id = name.to_camel_string();
        let field = DynNodeRefFieldType {
            name,
            store_id: data_field_id.into(),
            ref_node_type_id: node_type_id.clone(),
        };
        self.fields.insert(id, DynNodeFieldType::Ref(field));
    }
    pub fn define_connection(
        &mut self,
        name: impl Into<String>,
        edge_type_id: impl Into<DynEdgeTypeId>,
    ) {
        let name = Name::new_camel(name);
        let id = name.to_camel_string();
        let field = DynNodeConnectionFieldType {
            name,
            edge_type_id: edge_type_id.into(),
        };
        self.fields.insert(id, DynNodeFieldType::Connection(field));
    }
}

impl DynEdgeTypeBuilder {
    pub fn new(
        name: Name,
        tail: impl Into<DynNodeTypeId>,
        head: impl Into<DynNodeTypeId>,
    ) -> Self {
        Self {
            name,
            tail: tail.into(),
            head: head.into(),
            fields: Default::default(),
        }
    }

    pub fn define_data(
        &mut self,
        name: Name,
        data_field_id: impl Into<DynDataFieldId>,
        data_type_id: impl Into<DynDataTypeId>,
    ) {
        let id = name.to_camel_string();
        let field = DynDataFieldType {
            name,
            data_type_id: data_type_id.into(),
            store_data_field_id: data_field_id.into(),
        };
        self.fields.insert(id, field);
    }
}

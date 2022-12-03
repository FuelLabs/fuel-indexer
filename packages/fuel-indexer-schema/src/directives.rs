use crate::utils::IndexMethod;
pub struct Join {
    pub reference_field_name: String,
    pub field_name: String,
    pub field_type_name: String,
    pub object_name: String,
    pub reference_field_type_name: String,
}

impl Join {
    pub fn field_id(&self) -> String {
        format!("{}.{}", self.object_name, self.field_name)
    }
}

pub struct Unique(pub bool);

pub struct Index {
    pub column_name: String,
    pub method: IndexMethod,
}

impl Index {
    pub fn new(column_name: String) -> Self {
        Self {
            column_name,
            method: IndexMethod::default(),
        }
    }
}

use strum::{AsRefStr, EnumString};

/// Type of join to create between two tables.
pub struct Join {
    pub reference_field_name: String,
    pub field_name: String,
    pub field_type_name: String,
    pub object_name: String,
    pub reference_field_type_name: String,
}

impl Join {
    /// Get the fully qualitifed field name for this join.
    pub fn field_id(&self) -> String {
        format!("{}.{}", self.object_name, self.field_name)
    }
}

/// Unique index directive.
pub struct Unique(pub bool);

/// Index directive.
pub struct Index {
    pub column_name: String,
    pub method: IndexMethod,
}

/// Type of index to create on a table column.
#[derive(Debug, EnumString, AsRefStr, Default)]
pub enum IndexMethod {
    #[default]
    #[strum(serialize = "btree")]
    Btree,
    #[strum(serialize = "hash")]
    Hash,
}

impl Index {
    /// Create a new index directive.
    pub fn new(column_name: String) -> Self {
        Self {
            column_name,
            method: IndexMethod::default(),
        }
    }
}

/// Directive specifying to not build SQL tables for object.
pub struct NoRelation(pub bool);

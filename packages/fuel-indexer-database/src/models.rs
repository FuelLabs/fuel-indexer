use crate::DbType;
use std::fmt::Write;
use strum::{AsRefStr, EnumString};

pub trait CreateStatement {
    fn create_statement(&self) -> String;
}

#[derive(Debug)]
pub struct ColumnIndex {
    pub db_type: DbType,
    pub table_name: String,
    pub namespace: String,
    pub method: IndexMethod,
    pub unique: bool,
    pub column_name: String,
}

impl ColumnIndex {
    pub fn name(&self) -> String {
        format!("{}_{}_idx", &self.table_name, &self.column_name)
    }
}

impl CreateStatement for ColumnIndex {
    fn create_statement(&self) -> String {
        let mut frag = "CREATE ".to_string();
        if self.unique {
            frag += "UNIQUE ";
        }

        match self.db_type {
            DbType::Postgres => {
                let _ = write!(
                    frag,
                    "INDEX {} ON {}.{} USING {} ({});",
                    self.name(),
                    self.namespace,
                    self.table_name,
                    self.method.as_ref(),
                    self.column_name
                );
            }
            DbType::Sqlite => {
                let _ = write!(
                    frag,
                    "INDEX {} ON {}({});",
                    self.name(),
                    self.table_name,
                    self.column_name
                );
            }
        }

        frag
    }
}

#[derive(Debug, Clone, Copy, Default, EnumString, AsRefStr)]
pub enum OnDelete {
    #[default]
    #[strum(serialize = "NO ACTION")]
    NoAction,
    #[strum(serialize = "CASCADE")]
    Cascade,
    #[strum(serialize = "SET NULL")]
    SetNull,
}

#[derive(Debug, Clone, Copy, Default, EnumString, AsRefStr)]
pub enum OnUpdate {
    #[default]
    #[strum(serialize = "NO ACTION")]
    NoAction,
}

#[derive(Debug, Clone, Default)]
pub struct ForeignKey {
    pub db_type: DbType,
    pub namespace: String,
    pub table_name: String,
    pub column_name: String,
    pub reference_table_name: String,
    pub reference_column_name: String,
    pub reference_column_type: String,
    pub on_delete: OnDelete,
    pub on_update: OnUpdate,
}

impl ForeignKey {
    pub fn new(
        db_type: DbType,
        namespace: String,
        table_name: String,
        column_name: String,
        reference_table_name: String,
        ref_column_name: String,
        reference_column_type: String,
    ) -> Self {
        Self {
            db_type,
            namespace,
            table_name,
            column_name,
            reference_column_name: ref_column_name,
            reference_table_name,
            reference_column_type,
            ..Default::default()
        }
    }

    pub fn name(&self) -> String {
        format!(
            "fk_{}_{}__{}_{}",
            self.table_name,
            self.column_name,
            self.reference_table_name,
            self.reference_column_name
        )
    }
}

impl CreateStatement for ForeignKey {
    fn create_statement(&self) -> String {
        match self.db_type {
            DbType::Postgres => {
                format!(
                    "ALTER TABLE {}.{} ADD CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {}.{}({}) ON DELETE {} ON UPDATE {} INITIALLY DEFERRED;",
                    self.namespace,
                    self.table_name,
                    self.name(),
                    self.column_name,
                    self.namespace,
                    self.reference_table_name,
                    self.reference_column_name,
                    self.on_delete.as_ref(),
                    self.on_update.as_ref()
                )
            }
            DbType::Sqlite => {
                fn schema_type_to_sqlite_type(t: &str) -> String {
                    match t {
                        "ID" => "BIGINT".to_string(),
                        "UInt8" | "Int8" | "Int4" | "UInt4" => "INTEGER".to_string(),
                        _ => "TEXT".to_string(),
                    }
                }

                format!(
                    "ALTER TABLE {} DROP COLUMN {}; ALTER TABLE {} ADD COLUMN {} {} REFERENCES {}({});",
                    self.table_name,
                    self.column_name,
                    self.table_name,
                    self.column_name,
                    schema_type_to_sqlite_type(&self.reference_column_type),
                    self.reference_table_name,
                    self.reference_column_name,
                )
            }
        }
    }
}

#[derive(Debug, EnumString, AsRefStr, Default)]
pub enum IndexMethod {
    #[default]
    #[strum(serialize = "btree")]
    Btree,
    #[strum(serialize = "hash")]
    Hash,
}

pub struct IdCol {}
impl IdCol {
    pub fn to_lowercase_string() -> String {
        "id".to_string()
    }

    pub fn to_uppercase_string() -> String {
        "ID".to_string()
    }
}

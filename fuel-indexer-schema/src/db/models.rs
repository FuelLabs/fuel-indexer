use fuel_indexer_database::DbType;
use fuel_indexer_database_types::*;
// use sqlx::types::JsonValue;
use std::fmt::Write;

pub struct IdCol {}

impl IdCol {
    pub fn to_string() -> String {
        "id".to_string()
    }
}

#[derive(Debug)]
pub enum IndexMethod {
    Btree,
    Hash,
}

impl std::string::ToString for IndexMethod {
    fn to_string(&self) -> String {
        match self {
            IndexMethod::Btree => "btree".to_string(),
            IndexMethod::Hash => "hash".to_string(),
        }
    }
}

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
    pub column: NewColumn,
}

impl ColumnIndex {
    pub fn name(&self) -> String {
        format!("{}_{}_idx", &self.table_name, &self.column.column_name)
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
                    self.method.to_string(),
                    self.column.column_name
                );
            }
            DbType::Sqlite => {
                let _ = write!(
                    frag,
                    "INDEX {} ON {}({});",
                    self.name(),
                    self.table_name,
                    self.column.column_name
                );
            }
        }

        frag
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum OnDelete {
    #[default]
    NoAction,
    Cascade,
    SetNull,
}

impl std::string::ToString for OnDelete {
    fn to_string(&self) -> String {
        match self {
            OnDelete::NoAction => "NO ACTION".to_string(),
            OnDelete::Cascade => "CASCADE".to_string(),
            OnDelete::SetNull => "SET NULL".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum OnUpdate {
    #[default]
    NoAction,
}

impl std::string::ToString for OnUpdate {
    fn to_string(&self) -> String {
        match self {
            OnUpdate::NoAction => "NO ACTION".to_string(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ForeignKey {
    pub db_type: DbType,
    pub namespace: String,
    pub table_name: String,
    pub column_name: String,
    pub reference_table_name: String,
    pub reference_column_name: String,
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
        reference_column_name: String,
    ) -> Self {
        Self {
            db_type,
            namespace,
            table_name,
            column_name,
            reference_column_name,
            reference_table_name,
            ..Default::default()
        }
    }

    pub fn name(&self) -> String {
        format!(
            "fk_{}_{}",
            self.reference_table_name, self.reference_column_name
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
                    self.on_delete.to_string(),
                    self.on_update.to_string()
                )
            }
            DbType::Sqlite => {
                format!(
                    "ALTER TABLE {} DROP COLUMN {}; ALTER TABLE {} ADD COLUMN {} BIGINT REFERENCES {}({});",
                    self.table_name,
                    self.column_name,
                    self.table_name,
                    self.column_name,
                    self.reference_table_name,
                    self.reference_column_name,
                )
            }
        }
    }
}

use crate::db::IndexerConnection;
use std::fmt::Write;
use fuel_indexer_database_types::*;
use fuel_indexer_postgres as postgres;
use fuel_indexer_sqlite as sqlite;

pub async fn put_object(
    conn: &mut IndexerConnection,
    query: String,
    bytes: Vec<u8>,
) -> sqlx::Result<usize> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::put_object(c, query, bytes).await,
        IndexerConnection::Sqlite(ref mut c) => sqlite::put_object(c, query, bytes).await,
    }
}

pub async fn get_object(conn: &mut IndexerConnection, query: String) -> sqlx::Result<Vec<u8>> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::get_object(c, query).await,
        IndexerConnection::Sqlite(ref mut c) => sqlite::get_object(c, query).await,
    }
}

pub async fn run_query(conn: &mut IndexerConnection, query: String) -> sqlx::Result<String> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::run_query(c, query).await,
        IndexerConnection::Sqlite(ref mut c) => sqlite::run_query(c, query).await,
    }
}

pub async fn execute_query(conn: &mut IndexerConnection, query: String) -> sqlx::Result<usize> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::execute_query(c, query).await,
        IndexerConnection::Sqlite(ref mut c) => sqlite::execute_query(c, query).await,
    }
}

pub async fn root_columns_list_by_id(
    conn: &mut IndexerConnection,
    root_id: i64,
) -> sqlx::Result<Vec<RootColumns>> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::root_columns_list_by_id(c, root_id).await
        }
        IndexerConnection::Sqlite(ref mut c) => sqlite::root_columns_list_by_id(c, root_id).await,
    }
}

pub async fn new_root_columns(
    conn: &mut IndexerConnection,
    cols: Vec<NewRootColumns>,
) -> sqlx::Result<usize> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::new_root_columns(c, cols).await,
        IndexerConnection::Sqlite(ref mut c) => sqlite::new_root_columns(c, cols).await,
    }
}

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

        let _ = write!(
            frag,
            "INDEX {} ON {}.{} USING {} ({});",
            self.name(),
            self.namespace,
            self.table_name,
            self.method.to_string(),
            self.column.column_name
        );

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
        namespace: String,
        table_name: String,
        column_name: String,
        reference_table_name: String,
        reference_column_name: String,
    ) -> Self {
        Self {
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
}

pub async fn graph_root_latest(
    conn: &mut IndexerConnection,
    name: &str,
) -> sqlx::Result<GraphRoot> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::graph_root_latest(c, name).await,
        IndexerConnection::Sqlite(ref mut c) => sqlite::graph_root_latest(c, name).await,
    }
}

pub async fn new_graph_root(
    conn: &mut IndexerConnection,
    root: NewGraphRoot,
) -> sqlx::Result<usize> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::new_graph_root(c, root).await,
        IndexerConnection::Sqlite(ref mut c) => sqlite::new_graph_root(c, root).await,
    }
}

pub async fn type_id_list_by_name(
    conn: &mut IndexerConnection,
    name: &str,
    version: &str,
) -> sqlx::Result<Vec<TypeId>> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::type_id_list_by_name(c, name, version).await
        }
        IndexerConnection::Sqlite(ref mut c) => {
            sqlite::type_id_list_by_name(c, name, version).await
    }
}
}


pub async fn type_id_latest(
    conn: &mut IndexerConnection,
    schema_name: &str,
) -> sqlx::Result<String> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::type_id_latest(c, schema_name).await,
        IndexerConnection::Sqlite(ref mut c) => sqlite::type_id_latest(c, schema_name).await,
    }
}

pub async fn type_id_insert(
    conn: &mut IndexerConnection,
    type_ids: Vec<TypeId>,
) -> sqlx::Result<usize> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::type_id_insert(c, type_ids).await,
        IndexerConnection::Sqlite(ref mut c) => sqlite::type_id_insert(c, type_ids).await,
    }
}

pub async fn schema_exists(
    conn: &mut IndexerConnection,
    name: &str,
    version: &str,
) -> sqlx::Result<bool> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::schema_exists(c, name, version).await,
        IndexerConnection::Sqlite(ref mut c) => sqlite::schema_exists(c, name, version).await,
    }
}

pub async fn new_column_insert(
    conn: &mut IndexerConnection,
    cols: Vec<NewColumn>,
) -> sqlx::Result<usize> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::new_column_insert(c, cols).await,
        IndexerConnection::Sqlite(ref mut c) => sqlite::new_column_insert(c, cols).await,
    }
}

pub async fn list_column_by_id(
    conn: &mut IndexerConnection,
    col_id: i64,
) -> sqlx::Result<Vec<Columns>> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::list_column_by_id(c, col_id).await,
        IndexerConnection::Sqlite(ref mut c) => sqlite::list_column_by_id(c, col_id).await,
    }
}

pub async fn columns_get_schema(
    conn: &mut IndexerConnection,
    name: &str,
    version: &str,
) -> sqlx::Result<Vec<ColumnInfo>> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::columns_get_schema(c, name, version).await
        }
        IndexerConnection::Sqlite(ref mut c) => sqlite::columns_get_schema(c, name, version).await,
    }
}

use crate::{IndexerConfig, IndexerResult, Manifest};
use fuel_indexer_database::{queries, IndexerConnection, IndexerConnectionPool};
use fuel_indexer_lib::{
    fully_qualified_namespace, graphql::types::IdCol, utils::format_sql_query,
};
use fuel_indexer_schema::FtColumn;
use std::collections::HashMap;
use tracing::{debug, error, info};

/// Database for an executor instance, with schema info.
#[derive(Debug)]
pub struct Database {
    /// Connection pool for the database.
    pool: IndexerConnectionPool,

    /// Stashed connection for the current transaction.
    stashed: Option<IndexerConnection>,

    /// Namespace of the indexer.
    namespace: String,

    /// Identifier of the indexer.
    identifier: String,

    /// Version of the indexer.
    version: String,

    /// Table schema for the indexer.
    schema: HashMap<String, Vec<String>>,

    /// Mapping of `TypeId`s to tables.
    tables: HashMap<i64, String>,

    /// Indexer configuration.
    config: IndexerConfig,
}

// TODO: https://github.com/FuelLabs/fuel-indexer/issues/1139
unsafe impl Sync for Database {}
unsafe impl Send for Database {}

/// Check if the upsert query is for an ID column only.
fn is_id_only_upsert(columns: &[String]) -> bool {
    columns.len() == 2 && columns[0] == IdCol::to_lowercase_string()
}

impl Database {
    /// Create a new `Database`.
    pub async fn new(
        pool: IndexerConnectionPool,
        manifest: &Manifest,
        config: &IndexerConfig,
    ) -> Database {
        Database {
            pool,
            stashed: None,
            namespace: manifest.namespace().to_string(),
            identifier: manifest.identifier().to_string(),
            version: Default::default(),
            schema: Default::default(),
            tables: Default::default(),
            config: config.clone(),
        }
    }

    /// Open a database transaction.
    pub async fn start_transaction(&mut self) -> IndexerResult<usize> {
        let conn = self.pool.acquire().await?;
        self.stashed = Some(conn);
        debug!("Connection stashed as: {:?}", self.stashed);
        let conn = self.stashed.as_mut().expect(
            "No stashed connection for start transaction. Was a transaction started?",
        );
        let result = queries::start_transaction(conn).await?;
        Ok(result)
    }

    /// Commit transaction to database.
    pub async fn commit_transaction(&mut self) -> IndexerResult<usize> {
        let conn = self
            .stashed
            .as_mut()
            .expect("No stashed connection for commit. Was a transaction started?");
        let res = queries::commit_transaction(conn).await?;
        Ok(res)
    }

    /// Revert open transaction.
    pub async fn revert_transaction(&mut self) -> IndexerResult<usize> {
        let conn = self
            .stashed
            .as_mut()
            .expect("No stashed connection for revert. Was a transaction started?");
        let res = queries::revert_transaction(conn).await?;
        Ok(res)
    }

    /// Build an upsert query using a set of columns, insert values, update values, and a table name.
    fn upsert_query(
        &self,
        table: &str,
        columns: &[String],
        inserts: Vec<String>,
        updates: Vec<String>,
    ) -> String {
        if is_id_only_upsert(columns) {
            format!(
                "INSERT INTO {} ({}) VALUES ({}, $1::bytea) ON CONFLICT(id) DO NOTHING",
                table,
                columns.join(", "),
                inserts.join(", "),
            )
        } else {
            format!(
                "INSERT INTO {} ({}) VALUES ({}, $1::bytea) ON CONFLICT(id) DO UPDATE SET {}",
                table,
                columns.join(", "),
                inserts.join(", "),
                updates.join(", "),
            )
        }
    }

    /// Return a query to get an object from the database.
    fn get_query(&self, table: &str, object_id: u64) -> String {
        let q = format!("SELECT object from {table} where id = {object_id}");
        if self.config.verbose {
            info!("{q}");
        }
        q
    }

    /// Put an object into the database.
    pub async fn put_object(
        &mut self,
        type_id: i64,
        columns: Vec<FtColumn>,
        bytes: Vec<u8>,
    ) {
        let table = match self.tables.get(&type_id) {
            Some(t) => t,
            None => {
                error!(
                    r#"TypeId({type_id}) not found in tables: {:?}. 

Does the schema version in SchemaManager::new_schema match the schema version in Database::load_schema?

Do your WASM modules need to be rebuilt?

"#,
                    self.tables,
                );
                return;
            }
        };

        let inserts: Vec<_> = columns.iter().map(|col| col.query_fragment()).collect();
        let updates: Vec<_> = self.schema[table]
            .iter()
            .zip(columns.iter())
            .map(|(colname, value)| format!("{colname} = {}", value.query_fragment()))
            .collect();

        let columns = self.schema[table].clone();

        let query_text =
            format_sql_query(self.upsert_query(table, &columns, inserts, updates));

        let conn = self
            .stashed
            .as_mut()
            .expect("No stashed connection for put. Was a transaction started?");

        if self.config.verbose {
            info!("{query_text}");
        }

        if let Err(e) = queries::put_object(conn, query_text, bytes).await {
            error!("Failed to put_object: {e:?}");
        }
    }

    /// Get an object from the database.
    pub async fn get_object(&mut self, type_id: i64, object_id: u64) -> Option<Vec<u8>> {
        let table = &self.tables[&type_id];
        let query = self.get_query(table, object_id);
        let conn = self
            .stashed
            .as_mut()
            .expect("No stashed connection for get. Was a transaction started?");

        match queries::get_object(conn, query).await {
            Ok(v) => Some(v),
            Err(e) => {
                if let sqlx::Error::RowNotFound = e {
                    debug!("Row not found for object ID: {object_id}");
                } else {
                    error!("Failed to get_object: {e:?}");
                }
                None
            }
        }
    }

    /// Load the schema for this indexer from the database, and build a mapping of `TypeId`s to tables.
    pub async fn load_schema(&mut self, version: String) -> IndexerResult<()> {
        self.version = version;

        info!(
            "Loading schema for Indexer({}.{}) with Version({}).",
            self.namespace, self.identifier, self.version
        );

        let mut conn = self.pool.acquire().await?;
        let columns = queries::columns_get_schema(
            &mut conn,
            &self.namespace,
            &self.identifier,
            &self.version,
        )
        .await?;

        for column in columns {
            let table = &format!(
                "{}.{}",
                fully_qualified_namespace(&self.namespace, &self.identifier),
                &column.table_name
            );

            self.tables
                .entry(column.type_id)
                .or_insert_with(|| table.to_string());

            let columns = self
                .schema
                .entry(table.to_string())
                .or_insert_with(Vec::new);

            columns.push(column.column_name);
        }

        Ok(())
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn schema(&self) -> &HashMap<String, Vec<String>> {
        &self.schema
    }

    /// Put a record into the database.
    ///
    /// Specifically for many-to-many relationships.
    ///
    /// Since many-to-many relationships can _only_ ever reference certain `ID` fields
    /// on `TypeDefinition`s, we don't need to save any `FtColumn::Object` columns, which means
    /// we can simplify the `INSERT` into a simple string.
    ///
    /// There are multiple queries here because a single parent `TypeDefinition` can have several
    /// many-to-many relationships with children `TypeDefinition`s.
    pub async fn put_many_to_many_record(&mut self, queries: Vec<String>) {
        let conn = self
            .stashed
            .as_mut()
            .expect("No stashed connection for put. Was a transaction started?");

        for query in queries {
            if self.config.verbose {
                info!("{query}");
            }

            if let Err(e) = queries::put_many_to_many_record(conn, query).await {
                error!("Failed to put_many_to_many_record: {e:?}");
            }
        }
    }
}

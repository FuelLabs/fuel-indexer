use std::collections::HashMap;
use wasmer::Instance;
use crate::database::models::IdCol;

use crate::{ffi, IndexerError, IndexerResult, Manifest};
use fuel_indexer_schema::{
    db::{
        models,
        tables::{Schema, SchemaBuilder},
        IndexerConnection, IndexerConnectionPool,
    },
    schema_version, FtColumn,
};

/// Responsible for laying down graph schemas, processes schema upgrades.
pub struct SchemaManager {
    pool: IndexerConnectionPool,
}

impl SchemaManager {
    pub async fn new(db_conn: impl Into<String>) -> IndexerResult<SchemaManager> {
        let pool = IndexerConnectionPool::connect(&db_conn.into()).await?;

        Ok(SchemaManager { pool })
    }

    pub async fn new_schema(&self, name: &str, schema: &str) -> IndexerResult<()> {
        let mut connection = self.pool.acquire().await?;

        // TODO: Not doing much with version, but might be useful if we
        //       do graph schema upgrades
        let version = schema_version(schema);

        if !models::schema_exists(&mut connection, name, &version).await? {
            let _db_schema = SchemaBuilder::new(name, &version, self.pool.database_type())
                .build(schema)
                .commit_metadata(&mut connection)
                .await?;
        }
        Ok(())
    }

    pub async fn load_schema_wasm(&self, name: &str) -> IndexerResult<Schema> {
        // TODO: might be nice to cache this data in server?
        Ok(Schema::load_from_db(&self.pool, name).await?)
    }
}

/// Database for an executor instance, with schema info.
#[derive(Debug)]
pub struct Database {
    pub pool: IndexerConnectionPool,
    stashed: Option<IndexerConnection>,
    pub namespace: String,
    pub version: String,
    pub schema: HashMap<String, Vec<String>>,
    pub tables: HashMap<u64, String>,
}

// Hmm, TODO Mutecks instedddD
unsafe impl Sync for Database {}
unsafe impl Send for Database {}

impl Database {
    pub async fn new(db_conn: &str) -> IndexerResult<Database> {
        let pool = IndexerConnectionPool::connect(db_conn).await?;

        Ok(Database {
            pool,
            stashed: None,
            namespace: Default::default(),
            version: Default::default(),
            schema: Default::default(),
            tables: Default::default(),
        })
    }

    pub async fn start_transaction(&mut self) -> IndexerResult<usize> {
        let mut conn = self.pool.acquire().await?;
        let result = models::execute_query(&mut conn, "BEGIN".into()).await?;

        self.stashed = Some(conn);

        Ok(result)
    }

    pub async fn commit_transaction(&mut self) -> IndexerResult<usize> {
        let mut conn = self
            .stashed
            .take()
            .ok_or(IndexerError::NoTransactionError)?;
        Ok(models::execute_query(&mut conn, "COMMIT".into()).await?)
    }

    pub async fn revert_transaction(&mut self) -> IndexerResult<usize> {
        let mut conn = self
            .stashed
            .take()
            .ok_or(IndexerError::NoTransactionError)?;
        Ok(models::execute_query(&mut conn, "ROLLBACK".into()).await?)
    }

    fn upsert_query(
        &self,
        table: &str,
        columns: &[String],
        inserts: Vec<String>,
        updates: Vec<String>,
    ) -> String {
        let sql_table = self.pool.database_type().table_name(&self.namespace, table);

        format!(
            "INSERT INTO {}
                ({})
             VALUES
                ({}, $1)
             ON CONFLICT(id)
             DO UPDATE SET {}",
            sql_table,
            columns.join(", "),
            inserts.join(", "),
            updates.join(", "),
        )
    }

    fn get_query(&self, table: &str, object_id: u64) -> String {
        let sql_table = self.pool.database_type().table_name(&self.namespace, table);
        format!("SELECT object from {} where id = {}", sql_table, object_id)
    }

    pub async fn put_object(&mut self, type_id: u64, columns: Vec<FtColumn>, bytes: Vec<u8>) {
        let table = &self.tables[&type_id];
        let inserts: Vec<_> = columns.iter().map(|col| col.query_fragment()).collect();
        let updates: Vec<_> = self.schema[table]
            .iter()
            .zip(columns.iter())
            .filter_map(|(colname, value)| {
                if colname == &IdCol::to_string() {
                    None
                } else {
                    Some(format!("{} = {}", colname, value.query_fragment()))
                }
            })
            .collect();

        let columns = self.schema[table].clone();

        let query_text = self.upsert_query(table, &columns, inserts, updates);

        let conn = self
            .stashed
            .as_mut()
            .expect("No transaction has been opened!");
        let query = models::put_object(conn, query_text, bytes).await;

        query.expect("Query failed");
    }

    pub async fn get_object(&mut self, type_id: u64, object_id: u64) -> Option<Vec<u8>> {
        let table = &self.tables[&type_id];
        let query = self.get_query(table, object_id);

        let conn = self
            .stashed
            .as_mut()
            .expect("No transaction has been opened!");
        match models::get_object(conn, query).await {
            Ok(object) => Some(object),
            Err(sqlx::Error::RowNotFound) => None,
            e => {
                panic!("Error getting object! {:?}", e);
            }
        }
    }

    pub async fn load_schema_native(&mut self, manifest: Manifest) -> IndexerResult<()> {
        self.namespace = manifest.namespace;

        let mut conn = self.pool.acquire().await?;
        self.version = models::type_id_latest(&mut conn, &self.namespace).await?;

        let results = models::columns_get_schema(&mut conn, &self.namespace, &self.version).await?;

        for column in results {
            let table = &column.table_name;

            self.tables
                .entry(column.type_id as u64)
                .or_insert_with(|| table.to_string());

            let columns = self
                .schema
                .entry(table.to_string())
                .or_insert_with(Vec::new);

            columns.push(column.column_name);
        }

        Ok(())
    }

    pub async fn load_schema_wasm(&mut self, instance: &Instance) -> IndexerResult<()> {
        self.namespace = ffi::get_namespace(instance)?;
        self.version = ffi::get_version(instance)?;

        let mut conn = self.pool.acquire().await?;
        let results = models::columns_get_schema(&mut conn, &self.namespace, &self.version).await?;

        for column in results {
            let table = &column.table_name;

            self.tables
                .entry(column.type_id as u64)
                .or_insert_with(|| table.to_string());

            let columns = self
                .schema
                .entry(table.to_string())
                .or_insert_with(Vec::new);

            columns.push(column.column_name);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IndexEnv;
    use fuel_types::Address;
    use wasmer::{imports, Instance, Module, Store, WasmerEnv};
    use wasmer_compiler_cranelift::Cranelift;
    use wasmer_engine_universal::Universal;
    fn compiler() -> Cranelift {
        Cranelift::default()
    }

    // TODO: sqlite and postgres now....
    const GRAPHQL_SCHEMA: &str = include_str!("test_data/schema.graphql");
    const WASM_BYTES: &[u8] = include_bytes!("test_data/simple_wasm.wasm");
    const THING1_TYPE: u64 = 0xA21A262A00405632;
    const TEST_COLUMNS: [(&str, i32, &str); 7] = [
        ("thing2", 0, "id"),
        ("thing2", 1, "account"),
        ("thing2", 2, "hash"),
        ("thing2", 3, "object"),
        ("thing1", 0, "id"),
        ("thing1", 1, "account"),
        ("thing1", 2, "object"),
    ];

    async fn wasm_instance(database_url: &str) -> IndexerResult<Instance> {
        let compiler = compiler();
        let store = Store::new(&Universal::new(compiler).engine());
        let module = Module::new(&store, WASM_BYTES)?;

        let mut import_object = imports! {};

        let mut env = IndexEnv::new(database_url.to_string()).await?;
        let exports = ffi::get_exports(&env, &store);
        import_object.register("env", exports);

        let instance = Instance::new(&module, &import_object)?;
        env.init_with_instance(&instance)?;
        Ok(instance)
    }

    #[tokio::test]
    async fn test_schema_manager_postgres() {
        let database_url = "postgres://postgres:my-secret@127.0.0.1:5432";
        do_test(database_url).await;
    }

    #[tokio::test]
    async fn test_schema_manager_sqlite() {
        let workspace_root = env!("CARGO_MANIFEST_DIR");
        let database_url = format!("sqlite://{}/test.db", workspace_root);

        do_test(&database_url).await;
    }

    async fn do_test(database_url: &str) {
        let manager = SchemaManager::new(database_url)
            .await
            .expect("Could not create SchemaManager");

        let result = manager.new_schema("test_namespace", GRAPHQL_SCHEMA).await;
        println!("TJDEBUG {:?}", result);
        assert!(result.is_ok());

        let pool = IndexerConnectionPool::connect(database_url)
            .await
            .expect("Connection pool error");

        let version = schema_version(GRAPHQL_SCHEMA);
        let mut conn = pool.acquire().await.unwrap();
        let results = models::columns_get_schema(&mut conn, "test_namespace", &version)
            .await
            .expect("Metadata query failed");

        for (index, result) in results.into_iter().enumerate() {
            assert_eq!(result.table_name, TEST_COLUMNS[index].0);
            assert_eq!(result.column_position, TEST_COLUMNS[index].1);
            assert_eq!(result.column_name, TEST_COLUMNS[index].2);
        }

        let instance = wasm_instance(database_url)
            .await
            .expect("Error creating WASM module");

        let mut db = Database::new(database_url)
            .await
            .expect("Failed to create database object.");

        db.load_schema_wasm(&instance)
            .await
            .expect("Could not load db schema");

        assert_eq!(db.namespace, "test_namespace");
        assert_eq!(db.version, version);

        for column in TEST_COLUMNS.iter() {
            assert!(db.schema.contains_key(column.0));
        }

        let object_id = 4;
        let columns = vec![
            FtColumn::ID(object_id),
            FtColumn::Address(Address::from([0x04; 32])),
        ];
        let bytes = vec![0u8, 1u8, 2u8, 3u8];
        db.start_transaction()
            .await
            .expect("Start transaction failed");
        db.put_object(THING1_TYPE, columns, bytes.clone()).await;

        let obj = db.get_object(THING1_TYPE, object_id).await;
        assert!(obj.is_some());
        let obj = obj.unwrap();

        assert_eq!(obj, bytes);

        assert_eq!(db.get_object(THING1_TYPE, 90).await, None);
    }
}

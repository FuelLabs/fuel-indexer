use crate::{
    db::{
        tables::{Schema, SchemaBuilder},
        IndexerSchemaResult,
    },
    utils::{inject_native_entities_into_schema, schema_version},
};
use fuel_indexer_database::{queries, IndexerConnectionPool};

pub struct SchemaManager {
    pool: IndexerConnectionPool,
}

impl SchemaManager {
    pub async fn new(db_conn: impl Into<String>) -> IndexerSchemaResult<SchemaManager> {
        let pool = IndexerConnectionPool::connect(&db_conn.into()).await?;

        Ok(SchemaManager { pool })
    }

    pub async fn new_schema(&self, name: &str, schema: &str) -> IndexerSchemaResult<()> {
        // Schema is built in serveral different places so we add default entities here
        let schema = inject_native_entities_into_schema(schema);

        let mut connection = self.pool.acquire().await?;

        // TODO: Not doing much with version, but might be useful if we do graph schema upgrades
        let version = schema_version(&schema);

        if !queries::schema_exists(&mut connection, name, &version).await? {
            let _db_schema =
                SchemaBuilder::new(name, &version, self.pool.database_type())
                    .build(&schema)
                    .commit_metadata(&mut connection)
                    .await
                    .unwrap_or_else(|e| panic!("Failed to build new schema: {}", e));
        }
        Ok(())
    }

    pub async fn load_schema_wasm(&self, name: &str) -> IndexerSchemaResult<Schema> {
        // TODO: might be nice to cache this data in server?
        Ok(Schema::load_from_db(&self.pool, name).await?)
    }
}

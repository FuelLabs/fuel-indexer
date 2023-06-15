use crate::{
    db::tables::{Schema, SchemaBuilder},
    db::IndexerSchemaDbResult,
    utils::{inject_native_entities_into_schema, schema_version},
};
use fuel_indexer_database::{queries, IndexerConnection, IndexerConnectionPool};
use tracing::info;

pub struct SchemaManager {
    pool: IndexerConnectionPool,
}

impl SchemaManager {
    pub fn new(pool: IndexerConnectionPool) -> SchemaManager {
        SchemaManager { pool }
    }

    pub async fn new_schema(
        &self,
        namespace: &str,
        identifier: &str,
        schema: &str,
        conn: &mut IndexerConnection,
        is_native: bool,
    ) -> IndexerSchemaDbResult<()> {
        // Schema is built in serveral different places so we add default entities here
        let schema = inject_native_entities_into_schema(schema);

        // TODO: Not doing much with version, but might be useful if we do graph schema upgrades
        let version = schema_version(&schema);

        if !queries::schema_exists(conn, namespace, identifier, &version).await? {
            info!("Creating schema for Indexer({namespace}.{identifier}) with Version({version}).");
            let _db_schema = SchemaBuilder::new(
                namespace,
                identifier,
                &version,
                self.pool.database_type(),
                is_native,
            )?
            .build(&schema)?
            .commit_metadata(conn)
            .await?;
        }
        Ok(())
    }

    pub async fn load_schema(
        &self,
        namespace: &str,
        identifier: &str,
    ) -> IndexerSchemaDbResult<Schema> {
        Schema::load_from_db(&self.pool, namespace, identifier).await
    }
}

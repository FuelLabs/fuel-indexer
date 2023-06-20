use crate::{
    db::{tables::IndexerSchema, IndexerSchemaDbResult},
    utils::{inject_native_entities_into_schema, schema_version},
};
use fuel_indexer_database::{queries, IndexerConnection, IndexerConnectionPool};
use fuel_indexer_lib::ExecutionSource;
use fuel_indexer_types::graphql::GraphQLSchema;
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
        exec_source: ExecutionSource,
    ) -> IndexerSchemaDbResult<()> {
        let schema = GraphQLSchema::new(schema.to_string());
        let version = schema.version();

        if !queries::schema_exists(conn, namespace, identifier, version).await? {
            info!("Creating schema for Indexer({namespace}.{identifier}) with Version({version}).");
            let _db_schema = IndexerSchema::new(
                namespace,
                identifier,
                &schema,
                self.pool.database_type(),
                exec_source,
            )?
            .build(&schema)?
            .commit_sql_metadata(conn)
            .await?;
        }
        Ok(())
    }

    pub async fn load_schema(
        &self,
        namespace: &str,
        identifier: &str,
    ) -> IndexerSchemaDbResult<IndexerSchema> {
        // TODO: might be nice to cache this data in server?
        IndexerSchema::load_from_db(&self.pool, namespace, identifier).await
    }
}

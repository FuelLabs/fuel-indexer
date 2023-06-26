use crate::db::{tables::IndexerSchema, IndexerSchemaDbResult};
use fuel_indexer_database::{queries, IndexerConnection, IndexerConnectionPool};
use fuel_indexer_lib::{graphql::GraphQLSchema, ExecutionSource};
use tracing::info;

/// `SchemaManager` is a wrapper for `IndexerSchema` that also provides
/// stateful database connectivity.
pub struct SchemaManager {
    pool: IndexerConnectionPool,
}

impl SchemaManager {
    /// Create a new `SchemaManager`.
    pub fn new(pool: IndexerConnectionPool) -> SchemaManager {
        SchemaManager { pool }
    }

    /// Create a new schema for the given indexer.
    pub async fn new_schema(
        &self,
        namespace: &str,
        identifier: &str,
        schema: &str,
        exec_source: ExecutionSource,
        conn: &mut IndexerConnection,
    ) -> IndexerSchemaDbResult<()> {
        let schema = GraphQLSchema::new(schema.to_string());
        let version = schema.version();

        if !queries::schema_exists(conn, namespace, identifier, version).await? {
            info!("Creating schema for Indexer({namespace}.{identifier}) with Version({version}).");
            let _ = IndexerSchema::new(
                namespace,
                identifier,
                &schema,
                self.pool.database_type(),
                exec_source.clone(),
            )?
            .commit(&schema, exec_source, conn)
            .await?;
        }
        Ok(())
    }

    /// Load an existing schema for the given indexer.
    pub async fn load_schema(
        &self,
        namespace: &str,
        identifier: &str,
    ) -> IndexerSchemaDbResult<IndexerSchema> {
        // TODO: might be nice to cache this data in server?
        IndexerSchema::load(&self.pool, namespace, identifier).await
    }
}

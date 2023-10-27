//! # fuel_indexer_schema::db::manager
//!
//! A wrapper object that encapsulates `IndexerSchema` and provides stateful database
//! connectivity.

use crate::db::{tables::IndexerSchema, IndexerSchemaDbResult};
use fuel_indexer_database::{queries, IndexerConnection, IndexerConnectionPool};
use fuel_indexer_lib::graphql::GraphQLSchema;
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
        schema: GraphQLSchema,
        conn: &mut IndexerConnection,
    ) -> IndexerSchemaDbResult<()> {
        let version = schema.version();

        if !queries::schema_exists(conn, namespace, identifier, version).await? {
            info!("SchemaManager creating schema for Indexer({namespace}.{identifier}) with Version({version}).");
            let _ = IndexerSchema::new(
                namespace,
                identifier,
                &schema,
                self.pool.database_type(),
            )?
            .commit(&schema, conn)
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
        info!("SchemaManager loading schema for Indexer({namespace}.{identifier}).");
        IndexerSchema::load(&self.pool, namespace, identifier).await
    }
}

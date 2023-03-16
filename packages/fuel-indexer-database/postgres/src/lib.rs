#![deny(unused_crate_dependencies)]

use fuel_indexer_database_types::*;
use sqlx::{pool::PoolConnection, postgres::PgRow, types::JsonValue, Postgres, Row};
use tracing::info;

#[cfg(feature = "metrics")]
use fuel_indexer_metrics::METRICS;

const NONCE_EXPIRY: u64 = 3600; // 1 hour

pub async fn put_object(
    conn: &mut PoolConnection<Postgres>,
    query: String,
    bytes: Vec<u8>,
) -> sqlx::Result<usize> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.put_object_calls.inc();

    let mut builder = sqlx::QueryBuilder::new(query);

    let query = builder.build();
    let query = query.bind(bytes);
    let result = query.execute(conn).await?;

    Ok(result.rows_affected() as usize)
}

pub async fn get_object(
    conn: &mut PoolConnection<Postgres>,
    query: String,
) -> sqlx::Result<Vec<u8>> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.get_object_calls.inc();

    let mut builder = sqlx::QueryBuilder::new(query);

    let query = builder.build();

    let row = query.fetch_one(conn).await?;

    Ok(row.get(0))
}

pub async fn run_migration(conn: &mut PoolConnection<Postgres>) -> sqlx::Result<()> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.run_migration_calls.inc();

    sqlx::migrate!().run(conn).await?;

    Ok(())
}

pub async fn run_query(
    conn: &mut PoolConnection<Postgres>,
    query: String,
) -> sqlx::Result<JsonValue> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.run_query_calls.inc();

    let mut builder = sqlx::QueryBuilder::new(query);

    let query = builder.build();

    Ok(query
        .fetch_all(conn)
        .await?
        .iter()
        .map(|r| r.get::<'_, JsonValue, usize>(0))
        .collect())
}

pub async fn execute_query(
    conn: &mut PoolConnection<Postgres>,
    query: String,
) -> sqlx::Result<usize> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.execute_query_calls.inc();

    let mut builder = sqlx::QueryBuilder::new(query);

    let query = builder.build();

    let result = query.execute(conn).await?;

    Ok(result.rows_affected() as usize)
}

pub async fn root_columns_list_by_id(
    conn: &mut PoolConnection<Postgres>,
    root_id: i64,
) -> sqlx::Result<Vec<RootColumns>> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.root_columns_list_by_id_calls.inc();

    Ok(
        sqlx::query("SELECT * FROM graph_registry_root_columns WHERE root_id = $1")
            .bind(root_id)
            .fetch_all(conn)
            .await?
            .into_iter()
            .map(|row| {
                let id: i64 = row.get(0);
                let root_id: i64 = row.get(1);
                let column_name: String = row.get(2);
                let graphql_type: String = row.get(3);
                RootColumns {
                    id,
                    root_id,
                    column_name,
                    graphql_type,
                }
            })
            .collect::<Vec<RootColumns>>(),
    )
}

pub async fn new_root_columns(
    conn: &mut PoolConnection<Postgres>,
    cols: Vec<NewRootColumns>,
) -> sqlx::Result<usize> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.new_root_columns_calls.inc();

    let mut builder = sqlx::QueryBuilder::new(
        "INSERT INTO graph_registry_root_columns (root_id, column_name, graphql_type)",
    );

    builder.push_values(cols.into_iter(), |mut b, new_col| {
        b.push_bind(new_col.root_id)
            .push_bind(new_col.column_name)
            .push_bind(new_col.graphql_type);
    });

    let query = builder.build();

    let result = query.execute(conn).await?;

    Ok(result.rows_affected() as usize)
}

pub async fn new_graph_root(
    conn: &mut PoolConnection<Postgres>,
    root: NewGraphRoot,
) -> sqlx::Result<usize> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.new_graph_root_calls.inc();

    let mut builder = sqlx::QueryBuilder::new(
        "INSERT INTO graph_registry_graph_root (version, schema_name, schema_identifier, query, schema)",
    );

    builder.push_values(std::iter::once(root), |mut b, root| {
        b.push_bind(root.version)
            .push_bind(root.schema_name)
            .push_bind(root.schema_identifier)
            .push_bind(root.query)
            .push_bind(root.schema);
    });

    let query = builder.build();

    let result = query.execute(conn).await?;

    Ok(result.rows_affected() as usize)
}

pub async fn graph_root_latest(
    conn: &mut PoolConnection<Postgres>,
    namespace: &str,
    identifier: &str,
) -> sqlx::Result<GraphRoot> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.graph_root_latest_calls.inc();

    let row = sqlx::query(
        "SELECT * FROM graph_registry_graph_root
        WHERE schema_name = $1 AND schema_identifier = $2
        ORDER BY id DESC LIMIT 1",
    )
    .bind(namespace)
    .bind(identifier)
    .fetch_one(conn)
    .await?;

    let id: i64 = row.get(0);
    let version: String = row.get(1);
    let schema_name: String = row.get(2);
    let query: String = row.get(3);
    let schema: String = row.get(4);

    Ok(GraphRoot {
        id,
        version,
        schema_name,
        query,
        schema,
        schema_identifier: identifier.to_string(),
    })
}

pub async fn type_id_list_by_name(
    conn: &mut PoolConnection<Postgres>,
    namespace: &str,
    version: &str,
    identifier: &str,
) -> sqlx::Result<Vec<TypeId>> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.type_id_list_by_name_calls.inc();

    Ok(sqlx::query(
        "SELECT * FROM graph_registry_type_ids
        WHERE schema_name = $1 
        AND schema_version = $2 
        AND schema_identifier = $3",
    )
    .bind(namespace)
    .bind(version)
    .bind(identifier)
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| {
        let id: i64 = row.get(0);
        let schema_version: String = row.get(1);
        let schema_name: String = row.get(2);
        let graphql_name: String = row.get(3);
        let table_name: String = row.get(4);

        TypeId {
            id,
            schema_version,
            schema_name,
            table_name,
            graphql_name,
            schema_identifier: identifier.to_string(),
        }
    })
    .collect::<Vec<TypeId>>())
}

pub async fn type_id_latest(
    conn: &mut PoolConnection<Postgres>,
    schema_name: &str,
    identifier: &str,
) -> sqlx::Result<String> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.type_id_latest_calls.inc();

    let latest = sqlx::query(
        "SELECT schema_version FROM graph_registry_type_ids 
        WHERE schema_name = $1 
        AND schema_identifier = $2 
        ORDER BY id",
    )
    .bind(schema_name)
    .bind(identifier)
    .fetch_one(conn)
    .await?;

    let schema_version: String = latest.get(0);

    Ok(schema_version)
}

pub async fn type_id_insert(
    conn: &mut PoolConnection<Postgres>,
    type_ids: Vec<TypeId>,
) -> sqlx::Result<usize> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.type_id_insert_calls.inc();

    let mut builder = sqlx::QueryBuilder::new("INSERT INTO graph_registry_type_ids (id, schema_version, schema_name, schema_identifier, graphql_name, table_name)");

    builder.push_values(type_ids.into_iter(), |mut b, tid| {
        b.push_bind(tid.id)
            .push_bind(tid.schema_version)
            .push_bind(tid.schema_name)
            .push_bind(tid.schema_identifier)
            .push_bind(tid.graphql_name)
            .push_bind(tid.table_name);
    });

    let query = builder.build();

    let result = query.execute(conn).await?;

    Ok(result.rows_affected() as usize)
}

pub async fn schema_exists(
    conn: &mut PoolConnection<Postgres>,
    namespace: &str,
    identifier: &str,
    version: &str,
) -> sqlx::Result<bool> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.schema_exists_calls.inc();

    let count = sqlx::query(
        "SELECT COUNT(*) AS count FROM graph_registry_type_ids 
        WHERE schema_name = $1 
        AND schema_identifier = $2 
        AND schema_version = $3",
    )
    .bind(namespace)
    .bind(identifier)
    .bind(version)
    .fetch_one(conn)
    .await?;

    let count: i64 = count.get(0);

    Ok(count > 0)
}

pub async fn new_column_insert(
    conn: &mut PoolConnection<Postgres>,
    cols: Vec<NewColumn>,
) -> sqlx::Result<usize> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.new_column_insert_calls.inc();

    let mut builder = sqlx::QueryBuilder::new("INSERT INTO graph_registry_columns (type_id, column_position, column_name, column_type, nullable, graphql_type)");

    builder.push_values(cols.into_iter(), |mut b, new_col| {
        b.push_bind(new_col.type_id)
            .push_bind(new_col.column_position)
            .push_bind(new_col.column_name)
            .push_bind(new_col.column_type)
            .push_bind(new_col.nullable)
            .push_bind(new_col.graphql_type);
    });

    let query = builder.build();

    let result = query.execute(conn).await?;

    Ok(result.rows_affected() as usize)
}

pub async fn list_column_by_id(
    conn: &mut PoolConnection<Postgres>,
    col_id: i64,
) -> sqlx::Result<Vec<Columns>> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.list_column_by_id_calls.inc();

    Ok(
        sqlx::query("SELECT * FROM graph_registry_columns WHERE type_id = $1")
            .bind(col_id)
            .fetch_all(conn)
            .await?
            .into_iter()
            .map(|row| {
                let id: i64 = row.get(0);
                let type_id: i64 = row.get(1);
                let column_position: i32 = row.get(2);
                let column_name: String = row.get(3);
                let column_type: String = row.get(4);
                let nullable: bool = row.get(5);
                let graphql_type: String = row.get(6);

                Columns {
                    id,
                    type_id,
                    column_position,
                    column_name,
                    column_type,
                    nullable,
                    graphql_type,
                }
            })
            .collect::<Vec<Columns>>(),
    )
}

pub async fn columns_get_schema(
    conn: &mut PoolConnection<Postgres>,
    name: &str,
    identifier: &str,
    version: &str,
) -> sqlx::Result<Vec<ColumnInfo>> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.columns_get_schema_calls.inc();

    Ok(sqlx::query(
        "
            SELECT
            c.type_id as type_id,
            t.table_name as table_name,
            c.column_position as column_position,
            c.column_name as column_name,
            c.column_type as column_type
            FROM graph_registry_type_ids as t
            INNER JOIN graph_registry_columns as c ON t.id = c.type_id
            WHERE t.schema_name = $1 
            AND t.schema_identifier = $2 
            AND t.schema_version = $3
            ORDER BY c.type_id, c.column_position",
    )
    .bind(name)
    .bind(identifier)
    .bind(version)
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row: PgRow| {
        let type_id: i64 = row.get(0);
        let table_name: String = row.get(1);
        let column_position: i32 = row.get(2);
        let column_name: String = row.get(3);
        let column_type: String = row.get(4);

        ColumnInfo {
            type_id,
            table_name,
            column_position,
            column_name,
            column_type,
        }
    })
    .collect::<Vec<ColumnInfo>>())
}

pub async fn index_is_registered(
    conn: &mut PoolConnection<Postgres>,
    namespace: &str,
    identifier: &str,
) -> sqlx::Result<Option<RegisteredIndex>> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.index_is_registered_calls.inc();

    match sqlx::query(
        "SELECT * FROM index_registry 
        WHERE namespace = $1 
        AND identifier = $2",
    )
    .bind(namespace)
    .bind(identifier)
    .fetch_optional(conn)
    .await?
    {
        Some(row) => Ok(Some(RegisteredIndex {
            id: row.get(0),
            namespace: row.get(1),
            identifier: row.get(2),
            pubkey: row.get(3),
        })),
        None => Ok(None),
    }
}

pub async fn register_index(
    conn: &mut PoolConnection<Postgres>,
    namespace: &str,
    identifier: &str,
    pubkey: Option<&str>,
) -> sqlx::Result<RegisteredIndex> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.register_index_calls.inc();

    if let Some(index) = index_is_registered(conn, namespace, identifier).await? {
        return Ok(index);
    }

    let row = sqlx::query(
        "INSERT INTO index_registry (namespace, identifier)
         VALUES ($1, $2) 
         RETURNING id, namespace, identifier",
    )
    .bind(namespace)
    .bind(identifier)
    .fetch_one(conn)
    .await?;

    let id: i64 = row.get(0);
    let namespace: String = row.get(1);
    let identifier: String = row.get(2);
    let pubkey: Option<String> = row.get(3);

    Ok(RegisteredIndex {
        id,
        namespace,
        identifier,
        pubkey,
    })
}

pub async fn registered_indices(
    conn: &mut PoolConnection<Postgres>,
) -> sqlx::Result<Vec<RegisteredIndex>> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.registered_indices_calls.inc();

    Ok(sqlx::query("SELECT * FROM index_registry")
        .fetch_all(conn)
        .await?
        .into_iter()
        .map(|row| {
            let id: i64 = row.get(0);
            let namespace: String = row.get(1);
            let identifier: String = row.get(2);
            let pubkey = row.get(3);

            RegisteredIndex {
                id,
                namespace,
                identifier,
                pubkey,
            }
        })
        .collect::<Vec<RegisteredIndex>>())
}

pub async fn index_asset_version(
    conn: &mut PoolConnection<Postgres>,
    index_id: &i64,
    asset_type: &IndexAssetType,
) -> sqlx::Result<i64> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.index_asset_version_calls.inc();

    match sqlx::query(&format!(
        "SELECT COUNT(*) 
        FROM index_asset_registry_{} 
        WHERE index_id = {}",
        asset_type.as_ref(),
        index_id,
    ))
    .fetch_one(conn)
    .await
    {
        Ok(row) => Ok(row.try_get::<'_, i64, usize>(0).unwrap_or(0)),
        Err(_e) => Ok(0),
    }
}

pub async fn register_index_asset(
    conn: &mut PoolConnection<Postgres>,
    namespace: &str,
    identifier: &str,
    bytes: Vec<u8>,
    asset_type: IndexAssetType,
    pubkey: Option<&str>,
) -> sqlx::Result<IndexAsset> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.register_index_asset_calls.inc();

    let index = match index_is_registered(conn, namespace, identifier).await? {
        Some(index) => index,
        None => register_index(conn, namespace, identifier, pubkey).await?,
    };

    let digest = sha256_digest(&bytes);

    if let Some(asset) =
        asset_already_exists(conn, &asset_type, &bytes, &index.id).await?
    {
        info!(
            "Asset({asset_type:?}) for Index({}) already registered.",
            index.uid()
        );
        return Ok(asset);
    }

    let current_version = index_asset_version(conn, &index.id, &asset_type)
        .await
        .expect("Failed to get asset version.");

    let query = format!(
        "INSERT INTO index_asset_registry_{} (index_id, bytes, version, digest) VALUES ({}, $1, {}, '{digest}') RETURNING *",
        asset_type.as_ref(),
        index.id,
        current_version + 1,
    );

    let row = sqlx::QueryBuilder::new(query)
        .build()
        .bind(bytes)
        .fetch_one(conn)
        .await?;

    info!(
        "Registered Asset({:?}) to Index({}).",
        asset_type,
        index.uid()
    );

    let id = row.get(0);
    let index_id = row.get(1);
    let version = row.get(2);
    let digest = row.get(3);
    let bytes = row.get(4);

    Ok(IndexAsset {
        id,
        index_id,
        version,
        digest,
        bytes,
    })
}

pub async fn latest_asset_for_index(
    conn: &mut PoolConnection<Postgres>,
    index_id: &i64,
    asset_type: IndexAssetType,
) -> sqlx::Result<IndexAsset> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.latest_asset_for_index_calls.inc();

    let query = format!(
        "SELECT * FROM index_asset_registry_{} WHERE index_id = {} ORDER BY id DESC LIMIT 1",
        asset_type.as_ref(),
        index_id
    );

    let row = sqlx::query(&query).fetch_one(conn).await?;

    let id = row.get(0);
    let index_id = row.get(1);
    let version = row.get(2);
    let digest = row.get(3);
    let bytes = row.get(4);

    Ok(IndexAsset {
        id,
        index_id,
        version,
        digest,
        bytes,
    })
}

pub async fn latest_assets_for_index(
    conn: &mut PoolConnection<Postgres>,
    index_id: &i64,
) -> sqlx::Result<IndexAssetBundle> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.latest_assets_for_index_calls.inc();

    let wasm = latest_asset_for_index(conn, index_id, IndexAssetType::Wasm)
        .await
        .expect("Failed to retrieve wasm asset.");
    let schema = latest_asset_for_index(conn, index_id, IndexAssetType::Schema)
        .await
        .expect("Failed to retrieve schema asset.");
    let manifest = latest_asset_for_index(conn, index_id, IndexAssetType::Manifest)
        .await
        .expect("Failed to retrieve manifest asset.");

    Ok(IndexAssetBundle {
        wasm,
        schema,
        manifest,
    })
}

pub async fn last_block_height_for_indexer(
    conn: &mut PoolConnection<Postgres>,
    namespace: &str,
    identifier: &str,
) -> sqlx::Result<u64> {
    #[cfg(feature = "metrics")]
    METRICS
        .db
        .postgres
        .last_block_height_for_indexer_calls
        .inc();

    let query = format!(
        "SELECT MAX(id) FROM {namespace}_{identifier}.indexmetadataentity LIMIT 1"
    );

    let row = sqlx::query(&query).fetch_one(conn).await?;
    let id: i64 = match row.try_get(0) {
        Ok(id) => id,
        Err(_e) => return Ok(1),
    };

    Ok(id as u64)
}

// TODO: https://github.com/FuelLabs/fuel-indexer/issues/251
pub async fn asset_already_exists(
    conn: &mut PoolConnection<Postgres>,
    asset_type: &IndexAssetType,
    bytes: &Vec<u8>,
    index_id: &i64,
) -> sqlx::Result<Option<IndexAsset>> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.asset_already_exists_calls.inc();

    let digest = sha256_digest(bytes);

    let query = format!(
        "SELECT * FROM index_asset_registry_{} WHERE index_id = {} AND digest = '{}'",
        asset_type.as_ref(),
        index_id,
        digest
    );

    match sqlx::QueryBuilder::new(query).build().fetch_one(conn).await {
        Ok(row) => {
            let id = row.get(0);
            let index_id = row.get(1);
            let version = row.get(2);
            let digest = row.get(3);
            let bytes = row.get(4);

            Ok(Some(IndexAsset {
                id,
                index_id,
                version,
                digest,
                bytes,
            }))
        }
        Err(_e) => Ok(None),
    }
}

pub async fn index_id_for(
    conn: &mut PoolConnection<Postgres>,
    namespace: &str,
    identifier: &str,
) -> sqlx::Result<i64> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.index_id_for_calls.inc();

    let row = sqlx::query(
        "SELECT id FROM index_registry 
        WHERE namespace = $1 
        AND identifier = $2",
    )
    .bind(namespace)
    .bind(identifier)
    .fetch_one(conn)
    .await?;

    let id: i64 = row.get(0);

    Ok(id)
}

pub async fn penultimate_asset_for_index(
    conn: &mut PoolConnection<Postgres>,
    namespace: &str,
    identifier: &str,
    asset_type: IndexAssetType,
) -> sqlx::Result<IndexAsset> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.penultimate_asset_for_index_calls.inc();

    let index_id = index_id_for(conn, namespace, identifier).await?;
    let query = format!(
        "SELECT * FROM index_asset_registry_{} 
        WHERE index_id = {} ORDER BY id DESC LIMIT 1 OFFSET 1",
        asset_type.as_ref(),
        index_id
    );
    let row = sqlx::query(&query).fetch_one(conn).await?;

    let id = row.get(0);
    let index_id = row.get(1);
    let version = row.get(2);
    let digest = row.get(3);
    let bytes = row.get(4);

    Ok(IndexAsset {
        id,
        index_id,
        version,
        digest,
        bytes,
    })
}

pub async fn start_transaction(
    conn: &mut PoolConnection<Postgres>,
) -> sqlx::Result<usize> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.start_transaction_calls.inc();

    execute_query(conn, "BEGIN".into()).await
}

pub async fn commit_transaction(
    conn: &mut PoolConnection<Postgres>,
) -> sqlx::Result<usize> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.commit_transaction_calls.inc();

    execute_query(conn, "COMMIT".into()).await
}

pub async fn revert_transaction(
    conn: &mut PoolConnection<Postgres>,
) -> sqlx::Result<usize> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.revert_transaction_calls.inc();

    execute_query(conn, "ROLLBACK".into()).await
}

pub async fn remove_indexer(
    conn: &mut PoolConnection<Postgres>,
    namespace: &str,
    identifier: &str,
) -> sqlx::Result<()> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.remove_indexer.inc();

    let index_id = index_id_for(conn, namespace, identifier).await?;

    execute_query(
        conn,
        format!("DELETE FROM index_asset_registry_wasm WHERE index_id = {index_id}",),
    )
    .await?;

    execute_query(
        conn,
        format!("DELETE FROM index_asset_registry_manifest WHERE index_id = {index_id}",),
    )
    .await?;

    execute_query(
        conn,
        format!("DELETE FROM index_asset_registry_schema WHERE index_id = {index_id}",),
    )
    .await?;

    execute_query(
        conn,
        format!("DELETE FROM index_registry WHERE id = {index_id}",),
    )
    .await?;

    Ok(())
}

pub async fn remove_asset_by_version(
    conn: &mut PoolConnection<Postgres>,
    index_id: &i64,
    version: &i32,
    asset_type: IndexAssetType,
) -> sqlx::Result<()> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.remove_asset_by_version_calls.inc();

    execute_query(
        conn,
        format!(
            "DELETE FROM index_asset_registry_{0} WHERE index_id = {1} AND version = '{2}'",
            asset_type.as_ref(),
            index_id,
            version
        ),
    )
    .await?;

    Ok(())
}

pub async fn create_nonce(conn: &mut PoolConnection<Postgres>) -> sqlx::Result<Nonce> {
    let uid = uuid::Uuid::new_v4().as_simple().to_string();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let expiry = now + NONCE_EXPIRY;

    let row = sqlx::QueryBuilder::new(&format!(
        "INSERT INTO nonce (uid, expiry) VALUES ('{uid}', {expiry}) RETURNING *"
    ))
    .build()
    .fetch_one(conn)
    .await?;

    let uid: String = row.get(1);
    let expiry: i64 = row.get(2);

    Ok(Nonce { uid, expiry })
}

pub async fn delete_nonce(
    conn: &mut PoolConnection<Postgres>,
    nonce: &Nonce,
) -> sqlx::Result<()> {
    let _ = sqlx::query(&format!("DELETE FROM nonce WHERE uid = '{}'", nonce.uid))
        .execute(conn)
        .await?;

    Ok(())
}

pub async fn get_nonce(
    conn: &mut PoolConnection<Postgres>,
    uid: &str,
) -> sqlx::Result<Nonce> {
    let row = sqlx::query(&format!("SELECT * FROM nonce WHERE uid = '{uid}'"))
        .fetch_one(conn)
        .await?;

    let uid: String = row.get(1);
    let expiry: i64 = row.get(2);

    Ok(Nonce { uid, expiry })
}

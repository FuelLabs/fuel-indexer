#![deny(unused_crate_dependencies)]

use bigdecimal::ToPrimitive;
use fuel_indexer_database_types::*;
use fuel_indexer_lib::utils::sha256_digest;
use sqlx::{pool::PoolConnection, postgres::PgRow, types::JsonValue, Postgres, Row};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

#[cfg(feature = "metrics")]
use std::time::Instant;

#[cfg(feature = "metrics")]
use fuel_indexer_metrics::METRICS;

#[cfg(feature = "metrics")]
use fuel_indexer_macro_utils::metrics;

use chrono::{DateTime, NaiveDateTime, Utc};

/// The nonce expiry time in seconds.
const NONCE_EXPIRY_SECS: u64 = 3600;

/// Insert or update a blob of serialized `FtColumns` into the database.
#[cfg_attr(feature = "metrics", metrics)]
pub async fn put_object(
    conn: &mut PoolConnection<Postgres>,
    query: String,
    bytes: Vec<u8>,
) -> sqlx::Result<usize> {
    let mut builder = sqlx::QueryBuilder::new(query);
    let query = builder.build();
    let query = query.bind(bytes);
    let result = query.execute(conn).await?;
    Ok(result.rows_affected() as usize)
}

/// Fetch a blob of serialized `FtColumn`s from the database.
#[cfg_attr(feature = "metrics", metrics)]
pub async fn get_object(
    conn: &mut PoolConnection<Postgres>,
    query: String,
) -> sqlx::Result<Vec<u8>> {
    let mut builder = sqlx::QueryBuilder::new(query);
    let query = builder.build();
    let row = query.fetch_one(conn).await?;
    Ok(row.get(0))
}

/// Run database migrations.
#[cfg_attr(feature = "metrics", metrics)]
pub async fn run_migration(conn: &mut PoolConnection<Postgres>) -> sqlx::Result<()> {
    sqlx::migrate!().run(conn).await?;
    Ok(())
}

/// Run an arbitrary query and fetch all results.
///
/// Note that if the results of the query can't be converted to `JsonValue`, this function
/// will return an empty results set.
#[cfg_attr(feature = "metrics", metrics)]
pub async fn run_query(
    conn: &mut PoolConnection<Postgres>,
    query: String,
) -> sqlx::Result<JsonValue> {
    let mut builder = sqlx::QueryBuilder::new(query);
    let query = builder.build();
    Ok(query
        .fetch_all(conn)
        .await?
        .iter()
        .filter_map(|r| match r.try_get::<JsonValue, usize>(0) {
            Ok(v) => Some(v),
            Err(_e) => None,
        })
        .collect())
}

/// Execute an arbitrary query using the `QueryBuilder`.
#[cfg_attr(feature = "metrics", metrics)]
pub async fn execute_query(
    conn: &mut PoolConnection<Postgres>,
    query: String,
) -> sqlx::Result<usize> {
    let mut builder = sqlx::QueryBuilder::new(query);
    let query = builder.build();
    let result = query.execute(conn).await?;
    Ok(result.rows_affected() as usize)
}

/// Return a set of `RootColumn`s associated with a given `GraphRoot`.
#[cfg_attr(feature = "metrics", metrics)]
pub async fn root_columns_list_by_id(
    conn: &mut PoolConnection<Postgres>,
    root_id: i64,
) -> sqlx::Result<Vec<RootColumn>> {
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
                RootColumn {
                    id,
                    root_id,
                    column_name,
                    graphql_type,
                }
            })
            .collect::<Vec<RootColumn>>(),
    )
}

/// Persist a set of new `RootColumn`s associated with a given `GraphRoot`, to the database.
#[cfg_attr(feature = "metrics", metrics)]
pub async fn new_root_columns(
    conn: &mut PoolConnection<Postgres>,
    cols: Vec<RootColumn>,
) -> sqlx::Result<usize> {
    let mut builder = sqlx::QueryBuilder::new(
        "INSERT INTO graph_registry_root_columns (root_id, column_name, graphql_type)",
    );

    builder.push_values(cols, |mut b, new_col| {
        b.push_bind(new_col.root_id)
            .push_bind(new_col.column_name)
            .push_bind(new_col.graphql_type);
    });

    let query = builder.build();
    let result = query.execute(conn).await?;

    Ok(result.rows_affected() as usize)
}

/// Persist a new `GraphRoot` to the database.
#[cfg_attr(feature = "metrics", metrics)]
pub async fn new_graph_root(
    conn: &mut PoolConnection<Postgres>,
    root: GraphRoot,
) -> sqlx::Result<usize> {
    let mut builder = sqlx::QueryBuilder::new(
        "INSERT INTO graph_registry_graph_root (version, schema_name, schema_identifier, schema)",
    );

    builder.push_values(std::iter::once(root), |mut b, root| {
        b.push_bind(root.version)
            .push_bind(root.schema_name)
            .push_bind(root.schema_identifier)
            .push_bind(root.schema);
    });

    let query = builder.build();
    let result = query.execute(conn).await?;
    Ok(result.rows_affected() as usize)
}

/// Return the latest `GraphRoot` for a given indexer.
#[cfg_attr(feature = "metrics", metrics)]
pub async fn graph_root_latest(
    conn: &mut PoolConnection<Postgres>,
    namespace: &str,
    identifier: &str,
) -> sqlx::Result<GraphRoot> {
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
    let schema: String = row.get(3);

    Ok(GraphRoot {
        id,
        version,
        schema_name,
        schema,
        schema_identifier: identifier.to_string(),
    })
}

/// Return the set of `TypeIds` associated with the given indexer.
#[cfg_attr(feature = "metrics", metrics)]
pub async fn type_id_list_by_name(
    conn: &mut PoolConnection<Postgres>,
    namespace: &str,
    version: &str,
    identifier: &str,
) -> sqlx::Result<Vec<TypeId>> {
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
        let version: String = row.get(1);
        let namespace: String = row.get(2);
        let graphql_name: String = row.get(3);
        let table_name: String = row.get(4);
        let identifier: String = row.get(5);

        TypeId {
            id,
            version,
            namespace,
            table_name,
            graphql_name,
            identifier,
        }
    })
    .collect::<Vec<TypeId>>())
}

/// Return the latest schema version for a given indexer.
#[cfg_attr(feature = "metrics", metrics)]
pub async fn type_id_latest(
    conn: &mut PoolConnection<Postgres>,
    schema_name: &str,
    identifier: &str,
) -> sqlx::Result<String> {
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

/// Persist a set of new `TypeIds` to the database.
#[cfg_attr(feature = "metrics", metrics)]
pub async fn type_id_insert(
    conn: &mut PoolConnection<Postgres>,
    type_ids: Vec<TypeId>,
) -> sqlx::Result<usize> {
    let mut builder = sqlx::QueryBuilder::new("INSERT INTO graph_registry_type_ids (id, schema_version, schema_name, schema_identifier, graphql_name, table_name)");

    builder.push_values(type_ids, |mut b, tid| {
        b.push_bind(tid.id)
            .push_bind(tid.version)
            .push_bind(tid.namespace)
            .push_bind(tid.identifier)
            .push_bind(tid.graphql_name)
            .push_bind(tid.table_name);
    });

    let query = builder.build();
    let result = query.execute(conn).await?;
    Ok(result.rows_affected() as usize)
}

/// Indicate whether or not a given schema has been persisted to the database.
#[cfg_attr(feature = "metrics", metrics)]
pub async fn schema_exists(
    conn: &mut PoolConnection<Postgres>,
    namespace: &str,
    identifier: &str,
    version: &str,
) -> sqlx::Result<bool> {
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

/// Persist a set of new `Columns` to the database.
#[cfg_attr(feature = "metrics", metrics)]
pub async fn new_column_insert(
    conn: &mut PoolConnection<Postgres>,
    cols: Vec<Column>,
) -> sqlx::Result<usize> {
    let mut builder = sqlx::QueryBuilder::new("INSERT INTO graph_registry_columns (type_id, column_position, column_name, column_type, nullable, graphql_type, is_unique, persistence)");

    builder.push_values(cols, |mut b, new_col| {
        b.push_bind(new_col.type_id)
            .push_bind(new_col.position)
            .push_bind(new_col.name)
            .push_bind(new_col.coltype.to_string())
            .push_bind(new_col.nullable)
            .push_bind(new_col.graphql_type)
            .push_bind(new_col.unique)
            .push_bind(new_col.persistence.to_string());
    });

    let query = builder.build();

    let result = query.execute(conn).await?;

    Ok(result.rows_affected() as usize)
}

/// Return the set of `Columns` associated with a given `TypeId`.
#[cfg_attr(feature = "metrics", metrics)]
pub async fn list_column_by_id(
    conn: &mut PoolConnection<Postgres>,
    col_id: i64,
) -> sqlx::Result<Vec<Column>> {
    Ok(
        sqlx::query("SELECT * FROM graph_registry_columns WHERE type_id = $1")
            .bind(col_id)
            .fetch_all(conn)
            .await?
            .into_iter()
            .map(|row| {
                let id: i64 = row.get(0);
                let type_id: i64 = row.get(1);
                let position: i32 = row.get(2);
                let name: String = row.get(3);
                let coltype: String = row.get(4);
                let nullable: bool = row.get(5);
                let graphql_type: String = row.get(6);
                let unique: bool = row.get(7);
                let persistence: String = row.get(8);
                let array_coltype: Option<String> = row.get(9);

                Column {
                    id,
                    type_id,
                    position,
                    name,
                    coltype: ColumnType::from(coltype.as_str()),
                    nullable,
                    graphql_type,
                    unique,
                    persistence: Persistence::from_str(persistence.as_str())
                        .expect("Bad persistence."),
                    array_coltype: array_coltype.map(|t| ColumnType::from(t.as_str())),
                }
            })
            .collect::<Vec<Column>>(),
    )
}

/// Return a set of graph registry metadata (`ColumnInfo`) for a given indexer.
#[cfg_attr(feature = "metrics", metrics)]
pub async fn columns_get_schema(
    conn: &mut PoolConnection<Postgres>,
    name: &str,
    identifier: &str,
    version: &str,
) -> sqlx::Result<Vec<ColumnInfo>> {
    Ok(sqlx::query(
        "SELECT
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

/// Return the given indexer if it's already been registered.
#[cfg_attr(feature = "metrics", metrics)]
pub async fn get_indexer(
    conn: &mut PoolConnection<Postgres>,
    namespace: &str,
    identifier: &str,
) -> sqlx::Result<Option<RegisteredIndexer>> {
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
        Some(row) => {
            let created_at: DateTime<Utc> = {
                let created_at: NaiveDateTime = row.get(4);
                DateTime::<Utc>::from_utc(created_at, Utc)
            };

            Ok(Some(RegisteredIndexer {
                id: row.get(0),
                namespace: row.get(1),
                identifier: row.get(2),
                pubkey: row.get(3),
                created_at,
            }))
        }
        None => Ok(None),
    }
}

/// Register the given indexer's metadata.
///
/// Note that this only reigsters the indexer's metadata. Indexer assets are registered separately.
#[cfg_attr(feature = "metrics", metrics)]
pub async fn register_indexer(
    conn: &mut PoolConnection<Postgres>,
    namespace: &str,
    identifier: &str,
    pubkey: Option<&str>,
    created_at: DateTime<Utc>,
) -> sqlx::Result<RegisteredIndexer> {
    if let Some(index) = get_indexer(conn, namespace, identifier).await? {
        return Ok(index);
    }

    let row = sqlx::query(
        "INSERT INTO index_registry (namespace, identifier, pubkey, created_at)
         VALUES ($1, $2, $3, $4)
         RETURNING *",
    )
    .bind(namespace)
    .bind(identifier)
    .bind(pubkey)
    .bind(created_at)
    .fetch_one(conn)
    .await?;

    let id: i64 = row.get(0);
    let namespace: String = row.get(1);
    let identifier: String = row.get(2);
    let pubkey = row.get(3);
    let created_at: DateTime<Utc> = {
        let created_at: NaiveDateTime = row.get(4);
        DateTime::<Utc>::from_utc(created_at, Utc)
    };

    Ok(RegisteredIndexer {
        id,
        namespace,
        identifier,
        pubkey,
        created_at,
    })
}

pub async fn save_blockdata(
    conn: &mut PoolConnection<Postgres>,
    blockdata: &[fuel_indexer_types::fuel::BlockData],
) -> sqlx::Result<()> {
    if blockdata.is_empty() {
        return Ok(());
    }

    let mut query_string =
        "INSERT INTO index_block_data (block_height, block_data) VALUES ".to_string();
    for i in (0..blockdata.len() * 2).step_by(2) {
        query_string += &format!("(${}, ${}), ", i + 1, i + 2);
    }
    query_string.pop();
    query_string.pop();
    query_string += " ON CONFLICT DO NOTHING";

    let mut query = sqlx::query(&query_string);

    for bd in blockdata {
        let height = bd.height.to_i32().unwrap();
        let block = fuel_indexer_lib::utils::serialize(bd);
        query = query.bind(height).bind(block);
    }

    query.execute(conn).await?;

    Ok(())
}

/// Return all indexers registered to this indexer serivce.
#[cfg_attr(feature = "metrics", metrics)]
pub async fn all_registered_indexers(
    conn: &mut PoolConnection<Postgres>,
) -> sqlx::Result<Vec<RegisteredIndexer>> {
    Ok(sqlx::query("SELECT * FROM index_registry")
        .fetch_all(conn)
        .await?
        .into_iter()
        .map(|row| {
            let id: i64 = row.get(0);
            let namespace: String = row.get(1);
            let identifier: String = row.get(2);
            let pubkey = row.get(3);
            let created_at: DateTime<Utc> = {
                let created_at: NaiveDateTime = row.get(4);
                DateTime::<Utc>::from_utc(created_at, Utc)
            };

            RegisteredIndexer {
                id,
                namespace,
                identifier,
                pubkey,
                created_at,
            }
        })
        .collect::<Vec<RegisteredIndexer>>())
}

#[cfg_attr(feature = "metrics", metrics)]
pub async fn indexer_asset_version(
    conn: &mut PoolConnection<Postgres>,
    index_id: &i64,
    asset_type: &IndexerAssetType,
) -> sqlx::Result<i64> {
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
        Ok(row) => Ok(row.try_get::<i64, usize>(0).unwrap_or(0)),
        Err(_e) => Ok(0),
    }
}

/// Register a single indexer asset.
#[cfg_attr(feature = "metrics", metrics)]
pub async fn register_indexer_asset(
    conn: &mut PoolConnection<Postgres>,
    namespace: &str,
    identifier: &str,
    bytes: Vec<u8>,
    asset_type: IndexerAssetType,
    pubkey: Option<&str>,
) -> sqlx::Result<IndexerAsset> {
    let index = match get_indexer(conn, namespace, identifier).await? {
        Some(index) => index,
        None => {
            let created_at = DateTime::<Utc>::from(SystemTime::now());
            register_indexer(conn, namespace, identifier, pubkey, created_at).await?
        }
    };

    let digest = sha256_digest(&bytes);

    if let Some(asset) =
        asset_already_exists(conn, &asset_type, &bytes, &index.id).await?
    {
        info!(
            "Asset({asset_type:?}) for Indexer({}) already registered.",
            index.uid()
        );
        return Ok(asset);
    }

    let current_version = indexer_asset_version(conn, &index.id, &asset_type)
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
        "Registered Asset({:?}) with Version({}) to Indexer({}).",
        asset_type,
        digest,
        index.uid()
    );

    let id = row.get(0);
    let index_id = row.get(1);
    let version = row.get(2);
    let digest = row.get(3);
    let bytes = row.get(4);

    Ok(IndexerAsset {
        id,
        index_id,
        version,
        digest,
        bytes,
    })
}

/// Return the latest version for a given indexer asset type.
#[cfg_attr(feature = "metrics", metrics)]
pub async fn latest_asset_for_indexer(
    conn: &mut PoolConnection<Postgres>,
    index_id: &i64,
    asset_type: IndexerAssetType,
) -> sqlx::Result<IndexerAsset> {
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

    Ok(IndexerAsset {
        id,
        index_id,
        version,
        digest,
        bytes,
    })
}

/// Return the latest version for every indexer asset type.
#[cfg_attr(feature = "metrics", metrics)]
pub async fn latest_assets_for_indexer(
    conn: &mut PoolConnection<Postgres>,
    indexer_id: &i64,
) -> sqlx::Result<IndexerAssetBundle> {
    let wasm = latest_asset_for_indexer(conn, indexer_id, IndexerAssetType::Wasm).await?;
    let schema =
        latest_asset_for_indexer(conn, indexer_id, IndexerAssetType::Schema).await?;
    let manifest =
        latest_asset_for_indexer(conn, indexer_id, IndexerAssetType::Manifest).await?;

    Ok(IndexerAssetBundle {
        wasm,
        schema,
        manifest,
    })
}

/// Return the last block height that the given indexer has indexed.
#[cfg_attr(feature = "metrics", metrics)]
pub async fn last_block_height_for_indexer(
    conn: &mut PoolConnection<Postgres>,
    namespace: &str,
    identifier: &str,
) -> sqlx::Result<u32> {
    let query = format!(
        "SELECT MAX(block_height) FROM {namespace}_{identifier}.indexmetadataentity LIMIT 1"
    );

    let row = sqlx::query(&query).fetch_one(conn).await?;

    Ok(row
        .try_get::<i32, usize>(0)
        .map(|id| id.to_u32().expect("Bad block height."))
        .unwrap_or(0))
}

/// Return the last block height for stored blocks.
#[cfg_attr(feature = "metrics", metrics)]
pub async fn last_block_height_for_stored_blocks(
    conn: &mut PoolConnection<Postgres>,
) -> sqlx::Result<u32> {
    let query = "SELECT MAX(block_height) FROM index_block_data LIMIT 1".to_string();

    let row = sqlx::query(&query).fetch_one(conn).await?;

    let result = row
        .try_get::<i32, usize>(0)
        .map(|id| id.to_u32().expect("Bad block height."))
        .unwrap_or(0);

    Ok(result)
}

// TODO: https://github.com/FuelLabs/fuel-indexer/issues/251
#[cfg_attr(feature = "metrics", metrics)]
pub async fn asset_already_exists(
    conn: &mut PoolConnection<Postgres>,
    asset_type: &IndexerAssetType,
    bytes: &Vec<u8>,
    index_id: &i64,
) -> sqlx::Result<Option<IndexerAsset>> {
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

            Ok(Some(IndexerAsset {
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

/// Return the database ID for a given indexer.
#[cfg_attr(feature = "metrics", metrics)]
pub async fn get_indexer_id(
    conn: &mut PoolConnection<Postgres>,
    namespace: &str,
    identifier: &str,
) -> sqlx::Result<i64> {
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

/// Open a database transaction.
#[cfg_attr(feature = "metrics", metrics)]
pub async fn start_transaction(
    conn: &mut PoolConnection<Postgres>,
) -> sqlx::Result<usize> {
    execute_query(conn, "BEGIN".into()).await
}

/// Commit a database transaction.
#[cfg_attr(feature = "metrics", metrics)]
pub async fn commit_transaction(
    conn: &mut PoolConnection<Postgres>,
) -> sqlx::Result<usize> {
    execute_query(conn, "COMMIT".into()).await
}

/// Revert a database transaction.
#[cfg_attr(feature = "metrics", metrics)]
pub async fn revert_transaction(
    conn: &mut PoolConnection<Postgres>,
) -> sqlx::Result<usize> {
    execute_query(conn, "ROLLBACK".into()).await
}

/// Remove a given indexer.
///
/// This will also remove the given indexer's data if the caller specifies such.
#[cfg_attr(feature = "metrics", metrics)]
pub async fn remove_indexer(
    conn: &mut PoolConnection<Postgres>,
    namespace: &str,
    identifier: &str,
) -> sqlx::Result<()> {
    execute_query(
        conn,
        format!(
            "DELETE FROM index_asset_registry_wasm WHERE index_id IN
            (SELECT id FROM index_registry
                WHERE namespace = '{namespace}' AND identifier = '{identifier}')"
        ),
    )
    .await?;

    execute_query(
        conn,
        format!(
            "DELETE FROM index_asset_registry_manifest WHERE index_id IN
            (SELECT id FROM index_registry
                WHERE namespace = '{namespace}' AND identifier = '{identifier}')"
        ),
    )
    .await?;

    execute_query(
        conn,
        format!(
            "DELETE FROM index_asset_registry_schema WHERE index_id IN
            (SELECT id FROM index_registry
                WHERE namespace = '{namespace}' AND identifier = '{identifier}')"
        ),
    )
    .await?;

    execute_query(
        conn,
        format!(
            "DELETE FROM index_registry WHERE id IN
            (SELECT id FROM index_registry
                WHERE namespace = '{namespace}' AND identifier = '{identifier}')"
        ),
    )
    .await?;

    execute_query(
        conn,
        format!(
            "DELETE FROM graph_registry_columns WHERE type_id IN (SELECT id FROM graph_registry_type_ids WHERE schema_name = '{namespace}' AND schema_identifier = '{identifier}');"
        ),
    )
    .await?;

    execute_query(
        conn,
        format!(
            "DELETE FROM graph_registry_type_ids WHERE schema_name = '{namespace}' AND schema_identifier = '{identifier}';"
        ),
    )
    .await?;

    execute_query(
        conn,
        format!(
            "DELETE FROM graph_registry_root_columns WHERE root_id = (SELECT id FROM graph_registry_graph_root WHERE schema_name = '{namespace}' AND schema_identifier = '{identifier}');"
        ),
    )
    .await?;

    execute_query(
        conn,
        format!(
            "DELETE FROM graph_registry_graph_root WHERE schema_name = '{namespace}' AND schema_identifier = '{identifier}';"
        ),
    )
    .await?;

    execute_query(
        conn,
        format!("DROP SCHEMA {namespace}_{identifier} CASCADE"),
    )
    .await?;

    Ok(())
}

/// Create a new nonce for a requesting user's authentication.
#[cfg_attr(feature = "metrics", metrics)]
pub async fn create_nonce(conn: &mut PoolConnection<Postgres>) -> sqlx::Result<Nonce> {
    let uid = uuid::Uuid::new_v4().as_simple().to_string();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let expiry = now + NONCE_EXPIRY_SECS;

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

/// Return the specified nonce of a requesting user's authentication.
#[cfg_attr(feature = "metrics", metrics)]
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

/// Delete the specified nonce for a requesting user's authentication.
///
/// Happens after the user successfully authenticates.
#[cfg_attr(feature = "metrics", metrics)]
pub async fn delete_nonce(
    conn: &mut PoolConnection<Postgres>,
    nonce: &Nonce,
) -> sqlx::Result<()> {
    let _ = sqlx::query(&format!("DELETE FROM nonce WHERE uid = '{}'", nonce.uid))
        .execute(conn)
        .await?;

    Ok(())
}

/// Return whether or not the given user (identified by a public key) owns the given indexer.
#[cfg_attr(feature = "metrics", metrics)]
pub async fn indexer_owned_by(
    conn: &mut PoolConnection<Postgres>,
    namespace: &str,
    identifier: &str,
    pubkey: &str,
) -> sqlx::Result<()> {
    let row = sqlx::query(&format!("SELECT COUNT(*)::int FROM index_registry WHERE namespace = '{namespace}' AND identifier = '{identifier}' AND pubkey = '{pubkey}'"))
        .fetch_one(conn)
        .await?;

    let count = row.get::<i32, usize>(0);
    if count == 1 {
        return Ok(());
    }

    Err(sqlx::Error::RowNotFound)
}

/// Execute an arbitrary `INSERT` query where the content of the query includes
/// data for a many-to-many relationship.
#[cfg_attr(feature = "metrics", metrics)]
pub async fn put_many_to_many_record(
    conn: &mut PoolConnection<Postgres>,
    query: String,
) -> sqlx::Result<()> {
    execute_query(conn, query).await?;
    Ok(())
}

#![deny(unused_crate_dependencies)]

use fuel_indexer_database_types::*;
use fuel_indexer_lib::utils::sha256_digest;
use sqlx::{pool::PoolConnection, postgres::PgRow, types::JsonValue, Postgres, Row};
use tracing::info;

#[cfg(feature = "metrics")]
use fuel_indexer_metrics::METRICS;

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

    let query = format!("SELECT * FROM graph_registry_root_columns WHERE root_id = $1");

    let rows = sqlx::query(&query).bind(root_id).fetch_all(conn).await?;

    let root_columns: Vec<RootColumns> = rows
        .into_iter()
        .map(|row| RootColumns {
            id: row.get("id"),
            root_id: row.get("root_id"),
            column_name: row.get("column_name"),
            graphql_type: row.get("graphql_type"),
        })
        .collect();

    Ok(root_columns)
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

    let query = format!(
        "SELECT 
            id, 
            version, 
            schema_name, 
            query,
            schema, 
            schema_identifier
        FROM graph_registry_graph_root
        WHERE schema_name = $1 AND schema_identifier = $2
        ORDER BY id DESC LIMIT 1"
    );

    let row = sqlx::query(&query)
        .bind(namespace)
        .bind(identifier)
        .fetch_one(conn)
        .await?;

    Ok(GraphRoot {
        id: row.get("id"),
        version: row.get("version"),
        schema_name: row.get("schema_name"),
        query: row.get("query"),
        schema: row.get("schema"),
        schema_identifier: row.get("schema_identifier"),
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
    let query = format!(
        "SELECT 
            id, 
            schema_version, 
            schema_name, 
            table_name, 
            graphql_name, 
            schema_identifier
        FROM graph_registry_type_ids
        WHERE schema_name = $1 AND schema_version = $2 AND schema_identifier = $3"
    );

    let rows = sqlx::query(&query)
        .bind(namespace)
        .bind(version)
        .bind(identifier)
        .fetch_all(conn)
        .await?;

    let type_ids: Vec<TypeId> = rows
        .into_iter()
        .map(|row| TypeId {
            id: row.get("id"),
            schema_version: row.get("schema_version"),
            schema_name: row.get("schema_name"),
            table_name: row.get("table_name"),
            graphql_name: row.get("graphql_name"),
            schema_identifier: row.get("schema_identifier"),
        })
        .collect();

    Ok(type_ids)
}

pub async fn type_id_latest(
    conn: &mut PoolConnection<Postgres>,
    schema_name: &str,
    identifier: &str,
) -> sqlx::Result<String> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.type_id_latest_calls.inc();

    let latest = sqlx::query("SELECT schema_version FROM graph_registry_type_ids WHERE schema_name = $1 AND schema_identifier = $2 ORDER BY id")
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

    let count_query = "SELECT COUNT(*) AS count FROM graph_registry_type_ids WHERE schema_name = $1 AND schema_identifier = $2 AND schema_version = $3";
    let count = sqlx::query(count_query)
        .bind(namespace)
        .bind(identifier)
        .bind(version)
        .fetch_one(conn)
        .await?;

    let count: i64 = count.get("count");

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

    let query = "SELECT id, type_id, column_position, column_name, column_type, nullable, graphql_type FROM graph_registry_columns WHERE type_id = $1";
    let rows = sqlx::query(query).bind(col_id).fetch_all(conn).await?;

    let columns = rows
        .into_iter()
        .map(|row| Columns {
            id: row.get("id"),
            type_id: row.get("type_id"),
            column_position: row.get("column_position"),
            column_name: row.get("column_name"),
            column_type: row.get("column_type"),
            nullable: row.get("nullable"),
            graphql_type: row.get("graphql_type"),
        })
        .collect();

    Ok(columns)
}

pub async fn columns_get_schema(
    conn: &mut PoolConnection<Postgres>,
    name: &str,
    identifier: &str,
    version: &str,
) -> sqlx::Result<Vec<ColumnInfo>> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.columns_get_schema_calls.inc();

    let query = "
        SELECT
            c.type_id as type_id,
            t.table_name as table_name,
            c.column_position as column_position,
            c.column_name as column_name,
            c.column_type as column_type
        FROM graph_registry_type_ids as t
        INNER JOIN graph_registry_columns as c ON t.id = c.type_id
        WHERE t.schema_name = $1 AND t.schema_identifier = $2 AND t.schema_version = $3
        ORDER BY c.type_id, c.column_position
    ";

    let rows = sqlx::query(query)
        .bind(name)
        .bind(identifier)
        .bind(version)
        .map(|row: PgRow| ColumnInfo {
            type_id: row.get("type_id"),
            table_name: row.get("table_name"),
            column_position: row.get("column_position"),
            column_name: row.get("column_name"),
            column_type: row.get("column_type"),
        })
        .fetch_all(conn)
        .await?;

    Ok(rows)
}

pub async fn index_is_registered(
    conn: &mut PoolConnection<Postgres>,
    namespace: &str,
    identifier: &str,
) -> sqlx::Result<Option<RegisteredIndex>> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.index_is_registered_calls.inc();

    let query = "SELECT * FROM index_registry WHERE namespace = $1 AND identifier = $2";

    match sqlx::query(query)
        .bind(namespace)
        .bind(identifier)
        .fetch_optional(conn)
        .await?
    {
        Some(row) => Ok(Some(RegisteredIndex {
            id: row.get("id"),
            namespace: row.get("namespace"),
            identifier: row.get("identifier"),
        })),
        None => Ok(None),
    }
}

pub async fn register_index(
    conn: &mut PoolConnection<Postgres>,
    namespace: &str,
    identifier: &str,
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

    Ok(RegisteredIndex {
        id: row.get("id"),
        namespace: row.get("namespace"),
        identifier: row.get("identifier"),
    })
}

pub async fn registered_indices(
    conn: &mut PoolConnection<Postgres>,
) -> sqlx::Result<Vec<RegisteredIndex>> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.registered_indices_calls.inc();

    let query = "SELECT * FROM index_registry";
    let rows = sqlx::query(query).fetch_all(conn).await?;

    let registered_indices = rows
        .into_iter()
        .map(|row| RegisteredIndex {
            id: row.get("id"),
            namespace: row.get("namespace"),
            identifier: row.get("identifier"),
        })
        .collect();

    Ok(registered_indices)
}

pub async fn index_asset_version(
    conn: &mut PoolConnection<Postgres>,
    index_id: &i64,
    asset_type: &IndexAssetType,
) -> sqlx::Result<i64> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.index_asset_version_calls.inc();

    match sqlx::query(&format!(
        "SELECT COUNT(*) FROM index_asset_registry_{} WHERE index_id = {}",
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
) -> sqlx::Result<IndexAsset> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.register_index_asset_calls.inc();

    let index = match index_is_registered(conn, namespace, identifier).await? {
        Some(index) => index,
        None => register_index(conn, namespace, identifier).await?,
    };

    let digest = sha256_digest(&bytes);

    if let Some(asset) =
        asset_already_exists(conn, &asset_type, &bytes, &index.id).await?
    {
        info!(
            "Asset({:?}) for Index({}) already registered.",
            asset_type,
            index.uid()
        );
        return Ok(asset);
    }

    let current_version = index_asset_version(conn, &index.id, &asset_type)
        .await
        .expect("Failed to get asset version.");

    let query = format!(
        "INSERT INTO index_asset_registry_{} (index_id, bytes, version, digest) VALUES ({}, $1, {}, '{}') RETURNING *",
        asset_type.as_ref(),
        index.id,
        current_version + 1,
        digest,
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
        r#"select id from index_registry where namespace = $1 and identifier = $2"#,
    )
    .bind(namespace)
    .bind(identifier)
    .fetch_one(conn)
    .await?;

    let id: i64 = row.get("id");

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
        "SELECT * FROM index_asset_registry_{} WHERE index_id = {} ORDER BY id DESC LIMIT 1 OFFSET 1",
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

pub async fn remove_index(
    conn: &mut PoolConnection<Postgres>,
    namespace: &str,
    identifier: &str,
) -> sqlx::Result<()> {
    #[cfg(feature = "metrics")]
    METRICS.db.postgres.remove_index.inc();

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

use fuel_indexer_database_types::*;
use sqlx::{pool::PoolConnection, types::JsonValue, Connection, PgConnection, Postgres, Row};

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

pub async fn get_object(
    conn: &mut PoolConnection<Postgres>,
    query: String,
) -> sqlx::Result<Vec<u8>> {
    let mut builder = sqlx::QueryBuilder::new(query);

    let query = builder.build();

    let row = query.fetch_one(conn).await?;

    Ok(row.get(0))
}

pub async fn run_migration(database_url: &str) {
    let mut conn = PgConnection::connect(database_url)
        .await
        .expect("Failed to establish postgres connection.");
    sqlx::migrate!()
        .run(&mut conn)
        .await
        .expect("Failed postgres migration!");
}

pub async fn run_query(
    conn: &mut PoolConnection<Postgres>,
    query: String,
) -> sqlx::Result<JsonValue> {
    let mut builder = sqlx::QueryBuilder::new(query);

    let query = builder.build();

    let row = query.fetch_one(conn).await?;

    Ok(row.get::<'_, JsonValue, usize>(0))
}

pub async fn execute_query(
    conn: &mut PoolConnection<Postgres>,
    query: String,
) -> sqlx::Result<usize> {
    let mut builder = sqlx::QueryBuilder::new(query);

    let query = builder.build();

    let result = query.execute(conn).await?;

    Ok(result.rows_affected() as usize)
}

pub async fn root_columns_list_by_id(
    conn: &mut PoolConnection<Postgres>,
    root_id: i64,
) -> sqlx::Result<Vec<RootColumns>> {
    sqlx::query_as!(
        RootColumns,
        r#"SELECT
               id AS "id: i64", root_id AS "root_id: i64", column_name, graphql_type
           FROM graph_registry_root_columns
           WHERE root_id = $1"#,
        root_id
    )
    .fetch_all(conn)
    .await
}

pub async fn new_root_columns(
    conn: &mut PoolConnection<Postgres>,
    cols: Vec<NewRootColumns>,
) -> sqlx::Result<usize> {
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
    let mut builder = sqlx::QueryBuilder::new(
        "INSERT INTO graph_registry_graph_root (version, schema_name, query, schema)",
    );

    builder.push_values(std::iter::once(root), |mut b, root| {
        b.push_bind(root.version)
            .push_bind(root.schema_name)
            .push_bind(root.query)
            .push_bind(root.schema);
    });

    let query = builder.build();

    let result = query.execute(conn).await?;

    Ok(result.rows_affected() as usize)
}

pub async fn graph_root_latest(
    conn: &mut PoolConnection<Postgres>,
    name: &str,
) -> sqlx::Result<GraphRoot> {
    sqlx::query_as!(
        GraphRoot,
        "SELECT * FROM graph_registry_graph_root WHERE schema_name = $1 ORDER BY id DESC LIMIT 1",
        name
    )
    .fetch_one(conn)
    .await
}

pub async fn type_id_list_by_name(
    conn: &mut PoolConnection<Postgres>,
    name: &str,
    version: &str,
) -> sqlx::Result<Vec<TypeId>> {
    sqlx::query_as!(TypeId, "SELECT id, schema_version, schema_name, graphql_name, table_name FROM graph_registry_type_ids WHERE schema_name = $1 AND schema_version = $2", name, version).fetch_all(conn).await
}

pub async fn type_id_latest(
    conn: &mut PoolConnection<Postgres>,
    schema_name: &str,
) -> sqlx::Result<String> {
    let latest = sqlx::query_as!(
        IdLatest,
        "SELECT schema_version FROM graph_registry_type_ids WHERE schema_name = $1 ORDER BY id",
        schema_name
    )
    .fetch_one(conn)
    .await?;

    Ok(latest.schema_version)
}

pub async fn type_id_insert(
    conn: &mut PoolConnection<Postgres>,
    type_ids: Vec<TypeId>,
) -> sqlx::Result<usize> {
    let mut builder = sqlx::QueryBuilder::new("INSERT INTO graph_registry_type_ids (id, schema_version, schema_name, graphql_name, table_name)");

    builder.push_values(type_ids.into_iter(), |mut b, tid| {
        b.push_bind(tid.id)
            .push_bind(tid.schema_version)
            .push_bind(tid.schema_name)
            .push_bind(tid.graphql_name)
            .push_bind(tid.table_name);
    });

    let query = builder.build();

    let result = query.execute(conn).await?;

    Ok(result.rows_affected() as usize)
}

pub async fn schema_exists(
    conn: &mut PoolConnection<Postgres>,
    name: &str,
    version: &str,
) -> sqlx::Result<bool> {
    let versions = sqlx::query_as!(NumVersions, "SELECT count(*) as num FROM graph_registry_type_ids WHERE schema_name = $1 AND schema_version = $2", name, version).fetch_one(conn).await?;

    Ok(versions.num.is_some()
        && versions
            .num
            .expect("num field should be present in versions query results")
            > 0)
}

pub async fn new_column_insert(
    conn: &mut PoolConnection<Postgres>,
    cols: Vec<NewColumn>,
) -> sqlx::Result<usize> {
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
    sqlx::query_as!(Columns, r#"SELECT id AS "id: i64", type_id, column_position, column_name, column_type AS "column_type: String", nullable, graphql_type FROM graph_registry_columns WHERE type_id = $1"#, col_id).fetch_all(conn).await
}

pub async fn columns_get_schema(
    conn: &mut PoolConnection<Postgres>,
    name: &str,
    version: &str,
) -> sqlx::Result<Vec<ColumnInfo>> {
    sqlx::query_as!(
        ColumnInfo,
        r#"SELECT
               c.type_id as type_id,
               t.table_name as table_name,
               c.column_position as column_position,
               c.column_name as column_name,
               c.column_type as "column_type: String"
           FROM graph_registry_type_ids as t
           INNER JOIN graph_registry_columns as c
           ON t.id = c.type_id
           WHERE t.schema_name = $1
           AND t.schema_version = $2
           ORDER BY c.type_id, c.column_position"#,
        name,
        version
    )
    .fetch_all(conn)
    .await
}

pub async fn index_is_registered(
    conn: &mut PoolConnection<Postgres>,
    namespace: &str,
    identifier: &str,
) -> sqlx::Result<Option<RegisteredIndex>> {
    match sqlx::query_as!(
        RegisteredIndex,
        "SELECT * FROM index_registry WHERE namespace = $1 AND identifier = $2",
        namespace,
        identifier
    )
    .fetch_one(conn)
    .await
    {
        Ok(row) => Ok(Some(row)),
        Err(_e) => Ok(None),
    }
}

pub async fn register_index(
    conn: &mut PoolConnection<Postgres>,
    namespace: &str,
    identifier: &str,
) -> sqlx::Result<RegisteredIndex> {
    if let Some(index) = index_is_registered(conn, namespace, identifier).await? {
        return Ok(index);
    }

    let query = format!(
        r#"INSERT INTO index_registry (namespace, identifier) VALUES ('{}', '{}') RETURNING *"#,
        namespace, identifier,
    );

    let row = sqlx::QueryBuilder::new(query)
        .build()
        .fetch_one(conn)
        .await?;

    let id = row.get(0);
    let namespace = row.get(1);
    let identifier = row.get(2);

    Ok(RegisteredIndex {
        id,
        namespace,
        identifier,
    })
}

pub async fn registered_indices(
    conn: &mut PoolConnection<Postgres>,
) -> sqlx::Result<Vec<RegisteredIndex>> {
    sqlx::query_as!(RegisteredIndex, "SELECT * FROM index_registry",)
        .fetch_all(conn)
        .await
}

pub async fn index_asset_version(
    conn: &mut PoolConnection<Postgres>,
    namespace: &str,
    identifier: &str,
    asset_type: IndexAssetType,
) -> sqlx::Result<i64> {
    match sqlx::query(&format!(
        "SELECT COUNT(*) FROM index_asset_registry_{} WHERE namespace = '{}' AND identifier = '{}'",
        asset_type.to_string(),
        namespace,
        identifier
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
) -> sqlx::Result<()> {
    let index = match index_is_registered(conn, namespace, identifier).await? {
        Some(index) => index,
        None => register_index(conn, namespace, identifier).await?,
    };

    let current_version = index_asset_version(conn, namespace, identifier, asset_type.clone())
        .await
        .expect("Failed to get asset version.");

    let query = format!(
        "INSERT INTO index_asset_registry_{} (index_id, bytes, version) VALUES ({}, $1, {})",
        asset_type.to_string(),
        index.id,
        current_version + 1,
    );

    let _ = sqlx::QueryBuilder::new(query)
        .build()
        .bind(bytes)
        .execute(conn)
        .await;

    Ok(())
}

pub async fn latest_asset_for_index(
    conn: &mut PoolConnection<Postgres>,
    index_id: &i64,
    asset_type: IndexAssetType,
) -> sqlx::Result<IndexAsset> {
    let query = format!(
        "SELECT * FROM index_asset_registry_{} WHERE index_id = {} ORDER BY id DESC LIMIT 1",
        asset_type.to_string(),
        index_id
    );

    let row = sqlx::query(&query).fetch_one(conn).await?;

    let id = row.get(0);
    let index_id = row.get(1);
    let bytes = row.get(2);
    let version = row.get(3);

    Ok(IndexAsset {
        id,
        index_id,
        bytes,
        version,
    })
}

pub async fn latest_assets_for_index(
    conn: &mut PoolConnection<Postgres>,
    index_id: &i64,
) -> sqlx::Result<RegisteredIndexAssets> {
    let wasm = latest_asset_for_index(conn, index_id, IndexAssetType::Wasm)
        .await
        .expect("Failed to retrieve wasm asset.");
    let schema = latest_asset_for_index(conn, index_id, IndexAssetType::Schema)
        .await
        .expect("Failed to retrieve schema asset.");
    let manifest = latest_asset_for_index(conn, index_id, IndexAssetType::Manifest)
        .await
        .expect("Failed to retrieve manifest asset.");

    Ok(RegisteredIndexAssets {
        wasm,
        schema,
        manifest,
    })
}

pub async fn asset_already_exists(
    conn: &mut PoolConnection<Postgres>,
    asset_type: IndexAssetType,
    bytes: Vec<u8>,
    index_id: &i64,
) -> sqlx::Result<bool> {
    let query = format!(
        "SELECT * FROM index_asset_registry_{} WHERE index_id = {} AND bytes = $1",
        asset_type.to_string(),
        index_id
    );

    match sqlx::QueryBuilder::new(query)
        .build()
        .bind(bytes)
        .execute(conn)
        .await
    {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

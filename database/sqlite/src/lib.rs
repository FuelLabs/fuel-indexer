use fuel_indexer_database_types::*;
use sqlx::{pool::PoolConnection, Sqlite, SqliteConnection};
use sqlx::{Connection, Row};


pub async fn put_object(
    conn: &mut PoolConnection<Sqlite>,
    query: String,
    bytes: Vec<u8>,
) -> sqlx::Result<usize> {
    let mut builder = sqlx::QueryBuilder::new(query);

    let query = builder.build();
    let query = query.bind(bytes);
    let result = query.execute(conn).await?;

    Ok(result.rows_affected() as usize)
}

pub async fn get_object(conn: &mut PoolConnection<Sqlite>, query: String) -> sqlx::Result<Vec<u8>> {
    let mut builder = sqlx::QueryBuilder::new(query);

    let query = builder.build();

    let row = query.fetch_one(conn).await?;

    Ok(row.get(0))
}

pub async fn run_migration(database_url: &str) {
    let mut conn = SqliteConnection::connect(database_url)
        .await
        .expect("Failed to open sqlite database.");
    sqlx::migrate!()
        .run(&mut conn)
        .await
        .expect("Failed sqlite migration!");
}

pub async fn run_query(conn: &mut PoolConnection<Sqlite>, query: String) -> sqlx::Result<String> {
    let mut builder = sqlx::QueryBuilder::new(query);

    let query = builder.build();

    let row = query.fetch_one(conn).await?;

    Ok(row.get::<'_, String, usize>(0))
}

pub async fn execute_query(
    conn: &mut PoolConnection<Sqlite>,
    query: String,
) -> sqlx::Result<usize> {
    let mut builder = sqlx::QueryBuilder::new(query);

    let query = builder.build();

    let result = query.execute(conn).await?;

    Ok(result.rows_affected() as usize)
}

// NOTE: sqlx type inference is broken in sqlite, waiting for this to land, then we can clean this up:
//       https://github.com/launchbadge/sqlx/pull/1960
pub async fn root_columns_list_by_id(
    conn: &mut PoolConnection<Sqlite>,
    root_id: i64,
) -> sqlx::Result<Vec<RootColumns>> {
    let query = format!(
        r#"SELECT
               id, root_id, column_name, graphql_type
           FROM graph_registry_root_columns
           WHERE root_id = {}"#,
        root_id
    );
    let rows = sqlx::query(&query).fetch_all(conn).await?;

    let mut results = Vec::with_capacity(rows.len());
    for row in rows {
        let id = row.get(0);
        let root_id = row.get(1);
        let column_name = row.get(2);
        let graphql_type = row.get(3);

        results.push(RootColumns {
            id,
            root_id,
            column_name,
            graphql_type,
        });
    }

    Ok(results)
}

pub async fn new_root_columns(
    conn: &mut PoolConnection<Sqlite>,
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
    conn: &mut PoolConnection<Sqlite>,
    root: NewGraphRoot,
) -> sqlx::Result<usize> {
    let mut builder = sqlx::QueryBuilder::new("INSERT INTO graph_registry_graph_root (version, schema_name, query, schema)");

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
    conn: &mut PoolConnection<Sqlite>,
    name: &str,
) -> sqlx::Result<GraphRoot> {
    let query = format!(
        "SELECT * FROM graph_registry_graph_root WHERE schema_name = '{}' ORDER BY id DESC LIMIT 1",
        name
    );
    let row = sqlx::query(&query).fetch_one(conn).await?;

    let id = row.get(0);
    let version = row.get(1);
    let schema_name = row.get(2);
    let query = row.get(3);
    let schema = row.get(4);

    Ok(GraphRoot {
        id,
        version,
        schema_name,
        query,
        schema,
    })
}

pub async fn type_id_list_by_name(
    conn: &mut PoolConnection<Sqlite>,
    name: &str,
    version: &str,
) -> sqlx::Result<Vec<TypeId>> {
    let query = format!("SELECT id, schema_version, schema_name, graphql_name, table_name FROM graph_registry_type_ids WHERE schema_name = {} AND schema_version = {}", name, version);
    let rows = sqlx::query(&query).fetch_all(conn).await?;

    let mut results = Vec::with_capacity(rows.len());
    for row in rows {
        let id = row.get(0);
        let schema_version = row.get(1);
        let schema_name = row.get(2);
        let graphql_name = row.get(3);
        let table_name = row.get(4);

        results.push(TypeId {
            id,
            schema_version,
            schema_name,
            graphql_name,
            table_name,
        });
    }

    Ok(results)
}

pub async fn type_id_latest(
    conn: &mut PoolConnection<Sqlite>,
    schema_name: &str,
) -> sqlx::Result<String> {
    let query = format!(
        "SELECT schema_version FROM graph_registry_type_ids WHERE schema_name = '{}' ORDER BY id",
        schema_name
    );
    let row = sqlx::query(&query).fetch_one(conn).await?;

    Ok(row.get(0))
}

pub async fn type_id_insert(
    conn: &mut PoolConnection<Sqlite>,
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
    conn: &mut PoolConnection<Sqlite>,
    name: &str,
    version: &str,
) -> sqlx::Result<bool> {
    let query = format!("SELECT count(*) as num FROM graph_registry_type_ids WHERE schema_name = '{}' AND schema_version = '{}'", name, version);
    let row = sqlx::query(&query).fetch_one(conn).await?;

    let num: i64 = row.get(0);

    Ok(num > 0)
}

pub async fn new_column_insert(
    conn: &mut PoolConnection<Sqlite>,
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
    conn: &mut PoolConnection<Sqlite>,
    col_id: i64,
) -> sqlx::Result<Vec<Columns>> {
    let query = format!("SELECT id, type_id, column_position, column_name, column_type, nullable, graphql_type FROM graph_registry_columns WHERE type_id = {}", col_id);
    let rows = sqlx::query(&query).fetch_all(conn).await?;

    let mut results = Vec::with_capacity(rows.len());
    for row in rows {
        let id = row.get(0);
        let type_id = row.get(1);
        let column_position = row.get(3);
        let column_name = row.get(4);
        let column_type = row.get(5);
        let nullable = row.get(6);
        let graphql_type = row.get(7);

        results.push(Columns {
            id,
            type_id,
            column_position,
            column_name,
            column_type,
            nullable,
            graphql_type,
        });
    }

    Ok(results)
}

pub async fn columns_get_schema(
    conn: &mut PoolConnection<Sqlite>,
    name: &str,
    version: &str,
) -> sqlx::Result<Vec<ColumnInfo>> {
    let query = format!(
        r#"SELECT
               c.type_id as type_id,
               t.table_name as table_name,
               c.column_position as column_position,
               c.column_name as column_name,
               c.column_type as column_type
           FROM graph_registry_type_ids as t
           INNER JOIN graph_registry_columns as c
           ON t.id = c.type_id
           WHERE t.schema_name = '{}'
           AND t.schema_version = '{}'
           ORDER BY c.type_id, c.column_position"#,
        name, version
    );

    let rows = sqlx::query(&query).fetch_all(conn).await?;

    let mut results = Vec::with_capacity(rows.len());
    for row in rows {
        let type_id = row.get(0);
        let table_name = row.get(1);
        let column_position = row.get(2);
        let column_name = row.get(3);
        let column_type = row.get(4);

        results.push(ColumnInfo {
            type_id,
            table_name,
            column_position,
            column_name,
            column_type,
        });
    }

    Ok(results)
}

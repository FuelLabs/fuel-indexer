use sqlx::{Connection, Row};
use sqlx::{Sqlite, pool::PoolConnection, SqliteConnection};
use fuel_indexer_database_types::*;


pub async fn run_migration(database_url: &str) {
    let mut conn = SqliteConnection::connect(database_url).await.expect("Failed to open sqlite database.");
    sqlx::migrate!().run(&mut conn).await.expect("Failed sqlite migration!");
}


pub async fn run_query(conn: &mut PoolConnection<Sqlite>, query: String) -> sqlx::Result<String> {
    let mut builder = sqlx::QueryBuilder::new(query);

    let query = builder.build();

    let row = query.fetch_one(conn).await?;

    Ok(row.get::<'_, String, usize>(0))
}

pub async fn execute_query(conn: &mut PoolConnection<Sqlite>, query: String) -> sqlx::Result<usize> {
    let mut builder = sqlx::QueryBuilder::new(query);

    let query = builder.build();

    let result = query.execute(conn).await?;

    Ok(result.rows_affected() as usize)
}

pub async fn root_columns_list_by_id(_conn: &mut PoolConnection<Sqlite>, _root_id: i64) -> sqlx::Result<Vec<RootColumns>> {
    //sqlx::query_as!(RootColumns,
    //    r#"SELECT
    //           id, root_id, column_name, graphql_type
    //       FROM graph_registry_root_columns
    //       WHERE root_id = ?"#,
    //root_id).fetch_all(conn).await
    todo!()
}

pub async fn new_root_columns(_conn: &mut PoolConnection<Sqlite>, _cols: Vec<NewRootColumns>) -> sqlx::Result<usize> {
    todo!()
}

pub async fn new_graph_root(_conn: &mut PoolConnection<Sqlite>, _root: NewGraphRoot) -> sqlx::Result<usize> {
    todo!()
}

pub async fn graph_root_latest(_conn: &mut PoolConnection<Sqlite>, _name: &str) -> sqlx::Result<GraphRoot> {
    todo!()
}

pub async fn type_id_list_by_name(_conn: &mut PoolConnection<Sqlite>, _name: &str, _version: &str) -> sqlx::Result<Vec<TypeId>> {
    todo!()
}

pub async fn type_id_latest(_conn: &mut PoolConnection<Sqlite>, _schema_name: &str) -> sqlx::Result<String> {
    todo!()
}

pub async fn type_id_insert(_conn: &mut PoolConnection<Sqlite>, _type_ids: Vec<TypeId>) -> sqlx::Result<usize> {
    todo!()
}

pub async fn schema_exists(_conn: &mut PoolConnection<Sqlite>, _name: &str, _version: &str) -> sqlx::Result<bool> {
    todo!()
}

pub async fn new_column_insert(_conn: &mut PoolConnection<Sqlite>, _cols: Vec<NewColumn>) -> sqlx::Result<usize> {
    todo!()
}

pub async fn list_column_by_id(_conn: &mut PoolConnection<Sqlite>, _col_id: i64) -> sqlx::Result<Vec<Columns>> {
    todo!()
}

pub async fn columns_get_schema(_conn: &mut PoolConnection<Sqlite>, _name: &str, _version: &str) -> sqlx::Result<Vec<ColumnInfo>> {
    todo!()
}

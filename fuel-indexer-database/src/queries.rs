use crate::IndexerConnection;
use fuel_indexer_database_types::*;
use fuel_indexer_postgres as postgres;
use fuel_indexer_sqlite as sqlite;
use sqlx::types::JsonValue;
use std::str::FromStr;
use url::Url;

pub async fn graph_root_latest(
    conn: &mut IndexerConnection,
    name: &str,
) -> sqlx::Result<GraphRoot> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::graph_root_latest(c, name).await,
        IndexerConnection::Sqlite(ref mut c) => sqlite::graph_root_latest(c, name).await,
    }
}

pub async fn new_graph_root(
    conn: &mut IndexerConnection,
    root: NewGraphRoot,
) -> sqlx::Result<usize> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::new_graph_root(c, root).await,
        IndexerConnection::Sqlite(ref mut c) => sqlite::new_graph_root(c, root).await,
    }
}

pub async fn type_id_list_by_name(
    conn: &mut IndexerConnection,
    name: &str,
    version: &str,
) -> sqlx::Result<Vec<TypeId>> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::type_id_list_by_name(c, name, version).await
        }
        IndexerConnection::Sqlite(ref mut c) => {
            sqlite::type_id_list_by_name(c, name, version).await
        }
    }
}

pub async fn type_id_latest(
    conn: &mut IndexerConnection,
    schema_name: &str,
) -> sqlx::Result<String> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::type_id_latest(c, schema_name).await,
        IndexerConnection::Sqlite(ref mut c) => sqlite::type_id_latest(c, schema_name).await,
    }
}

pub async fn type_id_insert(
    conn: &mut IndexerConnection,
    type_ids: Vec<TypeId>,
) -> sqlx::Result<usize> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::type_id_insert(c, type_ids).await,
        IndexerConnection::Sqlite(ref mut c) => sqlite::type_id_insert(c, type_ids).await,
    }
}

pub async fn schema_exists(
    conn: &mut IndexerConnection,
    name: &str,
    version: &str,
) -> sqlx::Result<bool> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::schema_exists(c, name, version).await,
        IndexerConnection::Sqlite(ref mut c) => sqlite::schema_exists(c, name, version).await,
    }
}

pub async fn new_column_insert(
    conn: &mut IndexerConnection,
    cols: Vec<NewColumn>,
) -> sqlx::Result<usize> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::new_column_insert(c, cols).await,
        IndexerConnection::Sqlite(ref mut c) => sqlite::new_column_insert(c, cols).await,
    }
}

pub async fn list_column_by_id(
    conn: &mut IndexerConnection,
    col_id: i64,
) -> sqlx::Result<Vec<Columns>> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::list_column_by_id(c, col_id).await,
        IndexerConnection::Sqlite(ref mut c) => sqlite::list_column_by_id(c, col_id).await,
    }
}

pub async fn columns_get_schema(
    conn: &mut IndexerConnection,
    name: &str,
    version: &str,
) -> sqlx::Result<Vec<ColumnInfo>> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::columns_get_schema(c, name, version).await
        }
        IndexerConnection::Sqlite(ref mut c) => sqlite::columns_get_schema(c, name, version).await,
    }
}

pub async fn put_object(
    conn: &mut IndexerConnection,
    query: String,
    bytes: Vec<u8>,
) -> sqlx::Result<usize> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::put_object(c, query, bytes).await,
        IndexerConnection::Sqlite(ref mut c) => sqlite::put_object(c, query, bytes).await,
    }
}

pub async fn get_object(conn: &mut IndexerConnection, query: String) -> sqlx::Result<Vec<u8>> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::get_object(c, query).await,
        IndexerConnection::Sqlite(ref mut c) => sqlite::get_object(c, query).await,
    }
}

pub async fn run_query(conn: &mut IndexerConnection, query: String) -> sqlx::Result<JsonValue> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::run_query(c, query).await,
        IndexerConnection::Sqlite(ref mut c) => sqlite::run_query(c, query).await,
    }
}

pub async fn execute_query(conn: &mut IndexerConnection, query: String) -> sqlx::Result<usize> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::execute_query(c, query).await,
        IndexerConnection::Sqlite(ref mut c) => sqlite::execute_query(c, query).await,
    }
}

pub async fn root_columns_list_by_id(
    conn: &mut IndexerConnection,
    root_id: i64,
) -> sqlx::Result<Vec<RootColumns>> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::root_columns_list_by_id(c, root_id).await
        }
        IndexerConnection::Sqlite(ref mut c) => sqlite::root_columns_list_by_id(c, root_id).await,
    }
}

pub async fn new_root_columns(
    conn: &mut IndexerConnection,
    cols: Vec<NewRootColumns>,
) -> sqlx::Result<usize> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::new_root_columns(c, cols).await,
        IndexerConnection::Sqlite(ref mut c) => sqlite::new_root_columns(c, cols).await,
    }
}

pub async fn index_is_registered(
    conn: &mut IndexerConnection,
    namespace: &str,
    identifier: &str,
) -> sqlx::Result<Option<RegisteredIndex>> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::index_is_registered(c, namespace, identifier).await
        }
        IndexerConnection::Sqlite(ref mut c) => {
            sqlite::index_is_registered(c, namespace, identifier).await
        }
    }
}

pub async fn register_index(
    conn: &mut IndexerConnection,
    namespace: &str,
    identifier: &str,
) -> sqlx::Result<RegisteredIndex> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::register_index(c, namespace, identifier).await
        }
        IndexerConnection::Sqlite(ref mut c) => {
            sqlite::register_index(c, namespace, identifier).await
        }
    }
}

pub async fn registered_indices(
    conn: &mut IndexerConnection,
) -> sqlx::Result<Vec<RegisteredIndex>> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::registered_indices(c).await,
        IndexerConnection::Sqlite(ref mut c) => sqlite::registered_indices(c).await,
    }
}

pub async fn index_asset_version(
    conn: &mut IndexerConnection,
    index_id: &i64,
    asset_type: &IndexAssetType,
) -> sqlx::Result<i64> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::index_asset_version(c, index_id, asset_type).await
        }
        IndexerConnection::Sqlite(ref mut c) => {
            sqlite::index_asset_version(c, index_id, asset_type).await
        }
    }
}

pub async fn register_index_asset(
    conn: &mut IndexerConnection,
    namespace: &str,
    identifier: &str,
    bytes: Vec<u8>,
    asset_type: IndexAssetType,
) -> sqlx::Result<IndexAsset> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::register_index_asset(c, namespace, identifier, bytes, asset_type).await
        }
        IndexerConnection::Sqlite(ref mut c) => {
            sqlite::register_index_asset(c, namespace, identifier, bytes, asset_type).await
        }
    }
}

pub async fn latest_asset_for_index(
    conn: &mut IndexerConnection,
    index_id: &i64,
    asset_type: IndexAssetType,
) -> sqlx::Result<IndexAsset> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::latest_asset_for_index(c, index_id, asset_type).await
        }
        IndexerConnection::Sqlite(ref mut c) => {
            sqlite::latest_asset_for_index(c, index_id, asset_type).await
        }
    }
}

pub async fn latest_assets_for_index(
    conn: &mut IndexerConnection,
    index_id: &i64,
) -> sqlx::Result<IndexAssetBundle> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::latest_assets_for_index(c, index_id).await
        }
        IndexerConnection::Sqlite(ref mut c) => sqlite::latest_assets_for_index(c, index_id).await,
    }
}

pub async fn asset_already_exists(
    conn: &mut IndexerConnection,
    asset_type: &IndexAssetType,
    bytes: &Vec<u8>,
    index_id: &i64,
) -> sqlx::Result<Option<IndexAsset>> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::asset_already_exists(c, asset_type, bytes, index_id).await
        }
        IndexerConnection::Sqlite(ref mut c) => {
            sqlite::asset_already_exists(c, asset_type, bytes, index_id).await
        }
    }
}

pub async fn start_transaction(conn: &mut IndexerConnection) -> sqlx::Result<usize> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::start_transaction(c).await,
        IndexerConnection::Sqlite(ref mut c) => sqlite::start_transaction(c).await,
    }
}

pub async fn commit_transaction(conn: &mut IndexerConnection) -> sqlx::Result<usize> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::commit_transaction(c).await,
        IndexerConnection::Sqlite(ref mut c) => sqlite::commit_transaction(c).await,
    }
}

pub async fn revert_transaction(conn: &mut IndexerConnection) -> sqlx::Result<usize> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::revert_transaction(c).await,
        IndexerConnection::Sqlite(ref mut c) => sqlite::revert_transaction(c).await,
    }
}

pub async fn run_migration(database_url: &str) {
    let url = Url::from_str(database_url).expect("Database URL should be correctly formed");

    match url.scheme() {
        "postgres" => {
            postgres::run_migration(database_url).await;
        }
        "sqlite" => {
            sqlite::run_migration(database_url).await;
        }
        e => {
            panic!("database {} is not supported, use sqlite or postgres", e);
        }
    }
}

#[cfg(test)]
mod tests {}

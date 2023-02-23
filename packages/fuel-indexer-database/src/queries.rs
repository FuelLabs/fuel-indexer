use crate::{types::*, IndexerConnection};
use fuel_indexer_postgres as postgres;
use sqlx::types::JsonValue;

pub async fn graph_root_latest(
    conn: &mut IndexerConnection,
    namespace: &str,
    identifier: &str,
) -> sqlx::Result<GraphRoot> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::graph_root_latest(c, namespace, identifier).await
        }
   }
}

pub async fn new_graph_root(
    conn: &mut IndexerConnection,
    root: NewGraphRoot,
) -> sqlx::Result<usize> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::new_graph_root(c, root).await,
    }
}

pub async fn type_id_list_by_name(
    conn: &mut IndexerConnection,
    name: &str,
    version: &str,
    identifier: &str,
) -> sqlx::Result<Vec<TypeId>> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::type_id_list_by_name(c, name, version, identifier).await
        }
    }
}

pub async fn type_id_latest(
    conn: &mut IndexerConnection,
    schema_name: &str,
) -> sqlx::Result<String> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::type_id_latest(c, schema_name).await
        }
    }
}

pub async fn type_id_insert(
    conn: &mut IndexerConnection,
    type_ids: Vec<TypeId>,
) -> sqlx::Result<usize> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::type_id_insert(c, type_ids).await
        }
    }
}

pub async fn schema_exists(
    conn: &mut IndexerConnection,
    namespace: &str,
    identifier: &str,
    version: &str,
) -> sqlx::Result<bool> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::schema_exists(c, namespace, identifier, version).await
        }
    }
}

pub async fn new_column_insert(
    conn: &mut IndexerConnection,
    cols: Vec<NewColumn>,
) -> sqlx::Result<usize> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::new_column_insert(c, cols).await
        }
    }
}

pub async fn list_column_by_id(
    conn: &mut IndexerConnection,
    col_id: i64,
) -> sqlx::Result<Vec<Columns>> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::list_column_by_id(c, col_id).await
        }
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
    }
}

pub async fn put_object(
    conn: &mut IndexerConnection,
    query: String,
    bytes: Vec<u8>,
) -> sqlx::Result<usize> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::put_object(c, query, bytes).await
        }
    }
}

pub async fn get_object(
    conn: &mut IndexerConnection,
    query: String,
) -> sqlx::Result<Vec<u8>> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::get_object(c, query).await,
    }
}

pub async fn run_query(
    conn: &mut IndexerConnection,
    query: String,
) -> sqlx::Result<JsonValue> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::run_query(c, query).await,
    }
}

pub async fn execute_query(
    conn: &mut IndexerConnection,
    query: String,
) -> sqlx::Result<usize> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::execute_query(c, query).await,
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
    }
}

pub async fn new_root_columns(
    conn: &mut IndexerConnection,
    cols: Vec<NewRootColumns>,
) -> sqlx::Result<usize> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::new_root_columns(c, cols).await
        }
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
    }
}

pub async fn registered_indices(
    conn: &mut IndexerConnection,
) -> sqlx::Result<Vec<RegisteredIndex>> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::registered_indices(c).await,
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
            postgres::register_index_asset(c, namespace, identifier, bytes, asset_type)
                .await
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
    }
}

pub async fn index_id_for(
    conn: &mut IndexerConnection,
    namespace: &str,
    identifier: &str,
) -> sqlx::Result<i64> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::index_id_for(c, namespace, identifier).await
        }
    }
}

pub async fn start_transaction(conn: &mut IndexerConnection) -> sqlx::Result<usize> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::start_transaction(c).await,
    }
}

pub async fn commit_transaction(conn: &mut IndexerConnection) -> sqlx::Result<usize> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::commit_transaction(c).await,
    }
}

pub async fn revert_transaction(conn: &mut IndexerConnection) -> sqlx::Result<usize> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::revert_transaction(c).await,
    }
}

pub async fn run_migration(conn: &mut IndexerConnection) -> sqlx::Result<()> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::run_migration(c).await,
    }
}

pub async fn remove_index(
    conn: &mut IndexerConnection,
    namespace: &str,
    identifier: &str,
) -> sqlx::Result<()> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::remove_index(c, namespace, identifier).await
        }
    }
}

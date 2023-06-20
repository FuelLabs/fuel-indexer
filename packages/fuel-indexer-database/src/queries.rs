use crate::{types::*, IndexerConnection};
use fuel_indexer_postgres as postgres;
use sqlx::types::{
    chrono::{DateTime, Utc},
    JsonValue,
};

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
    identifier: &str,
) -> sqlx::Result<String> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::type_id_latest(c, schema_name, identifier).await
        }
    }
}

// REFACTOR - remove
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

pub async fn foo_type_id_insert(
    conn: &mut IndexerConnection,
    type_ids: Vec<FooTypeId>,
) -> sqlx::Result<usize> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::foo_type_id_insert(c, type_ids).await
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

// REFACTOR - remove
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

pub async fn foo_new_column_insert(
    conn: &mut IndexerConnection,
    cols: Vec<FooColumn>,
) -> sqlx::Result<usize> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::foo_new_column_insert(c, cols).await
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
    identifier: &str,
    version: &str,
) -> sqlx::Result<Vec<ColumnInfo>> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::columns_get_schema(c, name, identifier, version).await
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

pub async fn get_indexer(
    conn: &mut IndexerConnection,
    namespace: &str,
    identifier: &str,
) -> sqlx::Result<Option<RegisteredIndexer>> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::get_indexer(c, namespace, identifier).await
        }
    }
}

pub async fn register_indexer(
    conn: &mut IndexerConnection,
    namespace: &str,
    identifier: &str,
    pubkey: Option<&str>,
) -> sqlx::Result<RegisteredIndexer> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            let created_at = DateTime::<Utc>::from(std::time::SystemTime::now());
            postgres::register_indexer(c, namespace, identifier, pubkey, created_at).await
        }
    }
}

pub async fn all_registered_indexers(
    conn: &mut IndexerConnection,
) -> sqlx::Result<Vec<RegisteredIndexer>> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::all_registered_indexers(c).await
        }
    }
}

pub async fn indexer_asset_version(
    conn: &mut IndexerConnection,
    index_id: &i64,
    asset_type: &IndexerAssetType,
) -> sqlx::Result<i64> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::indexer_asset_version(c, index_id, asset_type).await
        }
    }
}

pub async fn register_indexer_asset(
    conn: &mut IndexerConnection,
    namespace: &str,
    identifier: &str,
    bytes: Vec<u8>,
    asset_type: IndexerAssetType,
    pubkey: Option<&str>,
) -> sqlx::Result<IndexerAsset> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::register_indexer_asset(
                c, namespace, identifier, bytes, asset_type, pubkey,
            )
            .await
        }
    }
}

pub async fn latest_asset_for_indexer(
    conn: &mut IndexerConnection,
    index_id: &i64,
    asset_type: IndexerAssetType,
) -> sqlx::Result<IndexerAsset> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::latest_asset_for_indexer(c, index_id, asset_type).await
        }
    }
}

pub async fn latest_assets_for_indexer(
    conn: &mut IndexerConnection,
    index_id: &i64,
) -> sqlx::Result<IndexerAssetBundle> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::latest_assets_for_indexer(c, index_id).await
        }
    }
}

pub async fn last_block_height_for_indexer(
    conn: &mut IndexerConnection,
    namespace: &str,
    identifier: &str,
) -> sqlx::Result<u32> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::last_block_height_for_indexer(c, namespace, identifier).await
        }
    }
}

pub async fn asset_already_exists(
    conn: &mut IndexerConnection,
    asset_type: &IndexerAssetType,
    bytes: &Vec<u8>,
    index_id: &i64,
) -> sqlx::Result<Option<IndexerAsset>> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::asset_already_exists(c, asset_type, bytes, index_id).await
        }
    }
}

pub async fn get_indexer_id(
    conn: &mut IndexerConnection,
    namespace: &str,
    identifier: &str,
) -> sqlx::Result<i64> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::get_indexer_id(c, namespace, identifier).await
        }
    }
}

pub async fn penultimate_asset_for_indexer(
    conn: &mut IndexerConnection,
    namespace: &str,
    identifier: &str,
    asset_type: IndexerAssetType,
) -> sqlx::Result<IndexerAsset> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::penultimate_asset_for_indexer(c, namespace, identifier, asset_type)
                .await
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

pub async fn remove_indexer(
    conn: &mut IndexerConnection,
    namespace: &str,
    identifier: &str,
) -> sqlx::Result<()> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::remove_indexer(c, namespace, identifier).await
        }
    }
}

pub async fn remove_latest_assets_for_indexer(
    conn: &mut IndexerConnection,
    namespace: &str,
    identifier: &str,
) -> sqlx::Result<()> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::remove_latest_assets_for_indexer(c, namespace, identifier).await
        }
    }
}

pub async fn remove_asset_by_version(
    conn: &mut IndexerConnection,
    index_id: &i64,
    version: &i32,
    asset_type: IndexerAssetType,
) -> sqlx::Result<()> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::remove_asset_by_version(c, index_id, version, asset_type).await
        }
    }
}

pub async fn create_nonce(conn: &mut IndexerConnection) -> sqlx::Result<Nonce> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::create_nonce(c).await,
    }
}

pub async fn get_nonce(conn: &mut IndexerConnection, uid: &str) -> sqlx::Result<Nonce> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::get_nonce(c, uid).await,
    }
}

pub async fn delete_nonce(
    conn: &mut IndexerConnection,
    nonce: &Nonce,
) -> sqlx::Result<()> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::delete_nonce(c, nonce).await,
    }
}

pub async fn indexer_owned_by(
    conn: &mut IndexerConnection,
    namespace: &str,
    identifier: &str,
    pubkey: &str,
) -> sqlx::Result<()> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::indexer_owned_by(c, namespace, identifier, pubkey).await
        }
    }
}

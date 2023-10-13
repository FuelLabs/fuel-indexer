use crate::{types::*, IndexerConnection};
use fuel_indexer_postgres as postgres;
use sqlx::types::{
    chrono::{DateTime, Utc},
    JsonValue,
};

/// Return the latest `GraphRoot` for a given indexer.
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

/// Persist a new `GraphRoot` to the database.
pub async fn new_graph_root(
    conn: &mut IndexerConnection,
    root: GraphRoot,
) -> sqlx::Result<usize> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::new_graph_root(c, root).await,
    }
}

/// Return the set of `TypeIds` associated with the given indexer.
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

/// Return the latest schema version for a given indexer.
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

/// Persist a set of new `TypeIds` to the database.
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

/// Indicate whether or not a given schema has been persisted to the database.
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

/// Persist a set of new `Columns` to the database.
pub async fn new_column_insert(
    conn: &mut IndexerConnection,
    cols: Vec<Column>,
) -> sqlx::Result<usize> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::new_column_insert(c, cols).await
        }
    }
}

/// Return the set of `Columns` associated with a given `TypeId`.
pub async fn list_column_by_id(
    conn: &mut IndexerConnection,
    col_id: i64,
) -> sqlx::Result<Vec<Column>> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::list_column_by_id(c, col_id).await
        }
    }
}

/// Return a set of graph registry metadata (`ColumnInfo`) for a given indexer.
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

/// Insert or update a blob of serialized `FtColumns` into the database.
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

/// Fetch a blob of serialized `FtColumns` from the database.
pub async fn get_object(
    conn: &mut IndexerConnection,
    query: String,
) -> sqlx::Result<Vec<u8>> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::get_object(c, query).await,
    }
}

/// Run an arbitrary query and fetch all results.
///
/// Note that if the results of the query can't be converted to `JsonValue`, this function
/// will return an empty results set.
pub async fn run_query(
    conn: &mut IndexerConnection,
    query: String,
) -> sqlx::Result<JsonValue> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::run_query(c, query).await,
    }
}

/// Execute an arbitrary query using the `QueryBuilder`.
pub async fn execute_query(
    conn: &mut IndexerConnection,
    query: String,
) -> sqlx::Result<usize> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::execute_query(c, query).await,
    }
}

/// Return a set of `RootColumn`s associated with a given `GraphRoot`.
pub async fn root_columns_list_by_id(
    conn: &mut IndexerConnection,
    root_id: i64,
) -> sqlx::Result<Vec<RootColumn>> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::root_columns_list_by_id(c, root_id).await
        }
    }
}

/// Persist a set of new `RootColumn`s associated with a given `GraphRoot`, to the database.
pub async fn new_root_columns(
    conn: &mut IndexerConnection,
    cols: Vec<RootColumn>,
) -> sqlx::Result<usize> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::new_root_columns(c, cols).await
        }
    }
}

/// Return the given indexer if it's already been registered.
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

/// Register the given indexer's metadata.
///
/// Note that this only reigsters the indexer's metadata. Indexer assets are registered separately.
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

/// Return all indexers registered to this indexer serivce.
pub async fn all_registered_indexers(
    conn: &mut IndexerConnection,
) -> sqlx::Result<Vec<RegisteredIndexer>> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::all_registered_indexers(c).await
        }
    }
}

/// Register a single indexer asset.
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

/// Returns the requested asset for an indexer with the given id.
pub async fn indexer_asset(
    conn: &mut IndexerConnection,
    index_id: &i64,
    asset_type: IndexerAssetType,
) -> sqlx::Result<IndexerAsset> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::indexer_asset(c, index_id, asset_type).await
        }
    }
}

/// Return every indexer asset type for an indexer with the give id.
pub async fn indexer_assets(
    conn: &mut IndexerConnection,
    index_id: &i64,
) -> sqlx::Result<IndexerAssetBundle> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::indexer_assets(c, index_id).await
        }
    }
}

/// Return the last block height that the given indexer has indexed.
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

/// Return the database ID for a given indexer.
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

/// Open a database transaction.
pub async fn start_transaction(conn: &mut IndexerConnection) -> sqlx::Result<usize> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::start_transaction(c).await,
    }
}

/// Commit a database transaction.
pub async fn commit_transaction(conn: &mut IndexerConnection) -> sqlx::Result<usize> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::commit_transaction(c).await,
    }
}

/// Revert a database transaction.
pub async fn revert_transaction(conn: &mut IndexerConnection) -> sqlx::Result<usize> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::revert_transaction(c).await,
    }
}

/// Run database migrations.
pub async fn run_migration(conn: &mut IndexerConnection) -> sqlx::Result<()> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::run_migration(c).await,
    }
}

/// Remove a given indexer.
///
/// This will also remove the given indexer's data if the caller specifies such.
pub async fn remove_indexer(
    conn: &mut IndexerConnection,
    namespace: &str,
    identifier: &str,
    remove_data: bool,
) -> sqlx::Result<()> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::remove_indexer(c, namespace, identifier, remove_data).await
        }
    }
}

/// Create a new nonce for a requesting user's authentication.
pub async fn create_nonce(conn: &mut IndexerConnection) -> sqlx::Result<Nonce> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::create_nonce(c).await,
    }
}

/// Return the specified nonce for a requesting user's authentication.
pub async fn get_nonce(conn: &mut IndexerConnection, uid: &str) -> sqlx::Result<Nonce> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::get_nonce(c, uid).await,
    }
}

/// Delete the specified nonce for a requesting user's authentication.
///
/// Happens after the user successfully authenticates.
pub async fn delete_nonce(
    conn: &mut IndexerConnection,
    nonce: &Nonce,
) -> sqlx::Result<()> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => postgres::delete_nonce(c, nonce).await,
    }
}

/// Return whether or not the given user (identified by a public key) owns the given indexer.
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

/// Execute an arbitrary `INSERT` query where the content of the query includes
/// data for a many-to-many relationship.
pub async fn put_many_to_many_record(
    conn: &mut IndexerConnection,
    query: String,
) -> sqlx::Result<()> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::put_many_to_many_record(c, query).await
        }
    }
}

pub async fn create_ensure_block_height_consecutive_trigger(
    conn: &mut IndexerConnection,
    namespace: &str,
    identifier: &str,
) -> sqlx::Result<()> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::create_ensure_block_height_consecutive_trigger(
                c, namespace, identifier,
            )
            .await
        }
    }
}

pub async fn remove_ensure_block_height_consecutive_trigger(
    conn: &mut IndexerConnection,
    namespace: &str,
    identifier: &str,
) -> sqlx::Result<()> {
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::remove_ensure_block_height_consecutive_trigger(
                c, namespace, identifier,
            )
            .await
        }
    }
}

pub use postgres::IndexerStatus;

pub async fn set_indexer_status(
    conn: &mut IndexerConnection,
    namespace: &str,
    identifier: &str,
    status: postgres::IndexerStatus,
    status_message: &str,
) -> sqlx::Result<()> {
    println!("Indexer({namespace}.{identifier}) status: {status:?}");
    match conn {
        IndexerConnection::Postgres(ref mut c) => {
            postgres::set_indexer_status(c, namespace, identifier, status, status_message)
                .await
        }
    }
}

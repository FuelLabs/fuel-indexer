use crate::core::Metric;
#[allow(unused)]
use prometheus::{self, register_int_counter, Counter, Encoder, IntCounter, TextEncoder};

#[derive(Clone, Debug)]
pub struct Queries {
    pub graph_root_latest: IntCounter,
    pub new_graph_root: IntCounter,
    pub type_id_list_by_name: IntCounter,
    pub type_id_latest: IntCounter,
    pub type_id_insert: IntCounter,
    pub schema_exists: IntCounter,
    pub new_column_insert: IntCounter,
    pub list_column_by_id: IntCounter,
    pub columns_get_schema: IntCounter,
    pub put_object: IntCounter,
    pub get_object: IntCounter,
    pub run_query: IntCounter,
    pub execute_query: IntCounter,
    pub root_columns_list_by_id: IntCounter,
    pub new_root_columns: IntCounter,
    pub index_is_registered: IntCounter,
    pub register_index: IntCounter,
    pub registered_indices: IntCounter,
    pub index_asset_version: IntCounter,
    pub register_index_asset: IntCounter,
    pub latest_asset_for_index: IntCounter,
    pub latest_assets_for_index: IntCounter,
    pub asset_already_exists: IntCounter,
    pub index_id_for: IntCounter,
    pub start_transaction: IntCounter,
    pub commit_transaction: IntCounter,
    pub revert_transaction: IntCounter,
    pub run_migration: IntCounter,
}

impl Metric for Queries {
    fn init() -> Self {
        Self {
            graph_root_latest: register_int_counter!(
                "graph_root_latest",
                "Count of calls to graph_root_latest."
            )
            .unwrap(),
            new_graph_root: register_int_counter!(
                "new_graph_root",
                "Count of calls to new_graph_root."
            )
            .unwrap(),
            type_id_list_by_name: register_int_counter!(
                "type_id_list_by_name",
                "Count of calls to type_id_list_by_name"
            )
            .unwrap(),
            type_id_latest: register_int_counter!(
                "type_id_latest",
                "Count of calls to type_id_latest."
            )
            .unwrap(),
            type_id_insert: register_int_counter!(
                "type_id_insert",
                "Count of calls to type_id_insert."
            )
            .unwrap(),
            schema_exists: register_int_counter!(
                "schema_exists",
                "Count of calls to schema_exists."
            )
            .unwrap(),
            new_column_insert: register_int_counter!(
                "new_column_insert",
                "Count of calls to new_column_insert."
            )
            .unwrap(),
            list_column_by_id: register_int_counter!(
                "list_column_by_id",
                "Count of calls to list_column_by_id."
            )
            .unwrap(),
            columns_get_schema: register_int_counter!(
                "columns_get_schema",
                "Count of calls to columns_get_schema."
            )
            .unwrap(),
            put_object: register_int_counter!(
                "put_object",
                "Count of calls to put_object."
            )
            .unwrap(),
            get_object: register_int_counter!(
                "get_object",
                "Count of calls to get_object."
            )
            .unwrap(),
            run_query: register_int_counter!("run_query", "Count of calls to run_query.")
                .unwrap(),
            execute_query: register_int_counter!(
                "execute_query",
                "Count of calls to execute_query."
            )
            .unwrap(),
            root_columns_list_by_id: register_int_counter!(
                "root_columns_list_by_id",
                "Count of calls to root_columns_list_by_id."
            )
            .unwrap(),
            new_root_columns: register_int_counter!(
                "new_root_columns",
                "Count of calls to new_root_columns."
            )
            .unwrap(),
            index_is_registered: register_int_counter!(
                "index_is_registered",
                "Count of calls to index_is_registered."
            )
            .unwrap(),
            register_index: register_int_counter!(
                "register_index",
                "Count of calls to register_index."
            )
            .unwrap(),
            registered_indices: register_int_counter!(
                "registered_indices",
                "Count of calls to registered_indices."
            )
            .unwrap(),
            index_asset_version: register_int_counter!(
                "index_asset_version",
                "Count of calls to index_asset_version."
            )
            .unwrap(),
            register_index_asset: register_int_counter!(
                "register_index_asset",
                "Count of calls to register_index_asset."
            )
            .unwrap(),
            latest_asset_for_index: register_int_counter!(
                "latest_asset_for_index",
                "Count of calls to latest_asset_for_index."
            )
            .unwrap(),
            latest_assets_for_index: register_int_counter!(
                "latest_assets_for_index",
                "Count of calls to latest_assets_for_index."
            )
            .unwrap(),
            asset_already_exists: register_int_counter!(
                "asset_already_exists",
                "Count of calls to asset_already_exists."
            )
            .unwrap(),
            index_id_for: register_int_counter!(
                "index_id_for",
                "Count of calls to index_id_for."
            )
            .unwrap(),
            start_transaction: register_int_counter!(
                "start_transaction",
                "Count of calls to start_transaction."
            )
            .unwrap(),
            commit_transaction: register_int_counter!(
                "commit_transaction",
                "Count of calls to commit_transaction."
            )
            .unwrap(),
            revert_transaction: register_int_counter!(
                "revert_transaction",
                "Count of calls to revert_transaction."
            )
            .unwrap(),
            run_migration: register_int_counter!(
                "run_migration",
                "Count of calls to run_migration."
            )
            .unwrap(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Database {
    pub write_ops: IntCounter,
    pub read_ops: IntCounter,
    pub bytes_written: IntCounter,
    pub bytes_read: IntCounter,
    pub queries: Queries,
}

impl Metric for Database {
    fn init() -> Self {
        Self {
            queries: Queries::init(),
            write_ops: register_int_counter!("write_ops", "Count of write operations.")
                .unwrap(),
            read_ops: register_int_counter!("read_ops", "Count of read operations.")
                .unwrap(),
            bytes_written: register_int_counter!(
                "bytes_written",
                "Total bytes written to the database."
            )
            .unwrap(),
            bytes_read: register_int_counter!(
                "bytes_read",
                "Total bytes read from the database."
            )
            .unwrap(),
        }
    }
}

use crate::core::Metric;
#[allow(unused)]
use prometheus::{self, register_int_counter, Counter, Encoder, IntCounter, TextEncoder};

#[derive(Clone, Debug)]
pub struct Queries {
    pub graph_root_latest_calls: IntCounter,
    pub new_graph_root_calls: IntCounter,
    pub type_id_list_by_name_calls: IntCounter,
    pub type_id_latest_calls: IntCounter,
    pub type_id_insert_calls: IntCounter,
    pub schema_exists_calls: IntCounter,
    pub new_column_insert_calls: IntCounter,
    pub list_column_by_id_calls: IntCounter,
    pub columns_get_schema_calls: IntCounter,
    pub put_object_calls: IntCounter,
    pub get_object_calls: IntCounter,
    pub run_query_calls: IntCounter,
    pub execute_query_calls: IntCounter,
    pub root_columns_list_by_id_calls: IntCounter,
    pub new_root_columns_calls: IntCounter,
    pub index_is_registered_calls: IntCounter,
    pub register_index_calls: IntCounter,
    pub registered_indices_calls: IntCounter,
    pub index_asset_version_calls: IntCounter,
    pub register_index_asset_calls: IntCounter,
    pub latest_asset_for_index_calls: IntCounter,
    pub latest_assets_for_index_calls: IntCounter,
    pub asset_already_exists_calls: IntCounter,
    pub index_id_for_calls: IntCounter,
    pub start_transaction_calls: IntCounter,
    pub commit_transaction_calls: IntCounter,
    pub revert_transaction_calls: IntCounter,
    pub run_migration_calls: IntCounter,
}

impl Metric for Queries {
    fn init() -> Self {
        Self {
            graph_root_latest_calls: register_int_counter!(
                "graph_root_latest_calls",
                "Count of calls to graph_root_latest_calls."
            )
            .unwrap(),
            new_graph_root_calls: register_int_counter!(
                "new_graph_root_calls",
                "Count of calls to new_graph_root_calls."
            )
            .unwrap(),
            type_id_list_by_name_calls: register_int_counter!(
                "type_id_list_by_name_calls",
                "Count of calls to type_id_list_by_name_calls"
            )
            .unwrap(),
            type_id_latest_calls: register_int_counter!(
                "type_id_latest_calls",
                "Count of calls to type_id_latest_calls."
            )
            .unwrap(),
            type_id_insert_calls: register_int_counter!(
                "type_id_insert_calls",
                "Count of calls to type_id_insert_calls."
            )
            .unwrap(),
            schema_exists_calls: register_int_counter!(
                "schema_exists_calls",
                "Count of calls to schema_exists_calls."
            )
            .unwrap(),
            new_column_insert_calls: register_int_counter!(
                "new_column_insert_calls",
                "Count of calls to new_column_insert_calls."
            )
            .unwrap(),
            list_column_by_id_calls: register_int_counter!(
                "list_column_by_id_calls",
                "Count of calls to list_column_by_id_calls."
            )
            .unwrap(),
            columns_get_schema_calls: register_int_counter!(
                "columns_get_schema_calls",
                "Count of calls to columns_get_schema_calls."
            )
            .unwrap(),
            put_object_calls: register_int_counter!(
                "put_object_calls",
                "Count of calls to put_object_calls."
            )
            .unwrap(),
            get_object_calls: register_int_counter!(
                "get_object_calls",
                "Count of calls to get_object_calls."
            )
            .unwrap(),
            run_query_calls: register_int_counter!(
                "run_query_calls",
                "Count of calls to run_query_calls."
            )
            .unwrap(),
            execute_query_calls: register_int_counter!(
                "execute_query_calls",
                "Count of calls to execute_query_calls."
            )
            .unwrap(),
            root_columns_list_by_id_calls: register_int_counter!(
                "root_columns_list_by_id_calls",
                "Count of calls to root_columns_list_by_id_calls."
            )
            .unwrap(),
            new_root_columns_calls: register_int_counter!(
                "new_root_columns_calls",
                "Count of calls to new_root_columns_calls."
            )
            .unwrap(),
            index_is_registered_calls: register_int_counter!(
                "index_is_registered_calls",
                "Count of calls to index_is_registered_calls."
            )
            .unwrap(),
            register_index_calls: register_int_counter!(
                "register_index_calls",
                "Count of calls to register_index_calls."
            )
            .unwrap(),
            registered_indices_calls: register_int_counter!(
                "registered_indices_calls",
                "Count of calls to registered_indices_calls."
            )
            .unwrap(),
            index_asset_version_calls: register_int_counter!(
                "index_asset_version_calls",
                "Count of calls to index_asset_version_calls."
            )
            .unwrap(),
            register_index_asset_calls: register_int_counter!(
                "register_index_asset_calls",
                "Count of calls to register_index_asset_calls."
            )
            .unwrap(),
            latest_asset_for_index_calls: register_int_counter!(
                "latest_asset_for_index_calls",
                "Count of calls to latest_asset_for_index_calls."
            )
            .unwrap(),
            latest_assets_for_index_calls: register_int_counter!(
                "latest_assets_for_index_calls",
                "Count of calls to latest_assets_for_index_calls."
            )
            .unwrap(),
            asset_already_exists_calls: register_int_counter!(
                "asset_already_exists_calls",
                "Count of calls to asset_already_exists_calls."
            )
            .unwrap(),
            index_id_for_calls: register_int_counter!(
                "index_id_for_calls",
                "Count of calls to index_id_for_calls."
            )
            .unwrap(),
            start_transaction_calls: register_int_counter!(
                "start_transaction_calls",
                "Count of calls to start_transaction_calls."
            )
            .unwrap(),
            commit_transaction_calls: register_int_counter!(
                "commit_transaction_calls",
                "Count of calls to commit_transaction_calls."
            )
            .unwrap(),
            revert_transaction_calls: register_int_counter!(
                "revert_transaction_calls",
                "Count of calls to revert_transaction_calls."
            )
            .unwrap(),
            run_migration_calls: register_int_counter!(
                "run_migration_calls",
                "Count of calls to run_migration_calls."
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

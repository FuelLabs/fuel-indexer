use crate::core::Metric;
#[allow(unused)]
use prometheus::{self, register_int_counter, Counter, Encoder, IntCounter, TextEncoder};

#[derive(Debug, Clone)]
pub struct SqliteQueries {
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
    pub remove_index: IntCounter,
}

impl Metric for SqliteQueries {
    fn init() -> Self {
        Self {
            graph_root_latest_calls: register_int_counter!(
                "sqlite_graph_root_latest_calls",
                "Count of calls to sqlite graph_root_latest_calls."
            )
            .unwrap(),
            new_graph_root_calls: register_int_counter!(
                "sqlite_new_graph_root_calls",
                "Count of calls to sqlite new_graph_root_calls."
            )
            .unwrap(),
            type_id_list_by_name_calls: register_int_counter!(
                "sqlite_type_id_list_by_name_calls",
                "Count of calls to sqlite type_id_list_by_name_calls"
            )
            .unwrap(),
            type_id_latest_calls: register_int_counter!(
                "sqlite_type_id_latest_calls",
                "Count of calls to sqlite type_id_latest_calls."
            )
            .unwrap(),
            type_id_insert_calls: register_int_counter!(
                "sqlite_type_id_insert_calls",
                "Count of calls to sqlite type_id_insert_calls."
            )
            .unwrap(),
            schema_exists_calls: register_int_counter!(
                "sqlite_schema_exists_calls",
                "Count of calls to sqlite schema_exists_calls."
            )
            .unwrap(),
            new_column_insert_calls: register_int_counter!(
                "sqlite_new_column_insert_calls",
                "Count of calls to sqlite new_column_insert_calls."
            )
            .unwrap(),
            list_column_by_id_calls: register_int_counter!(
                "sqlite_list_column_by_id_calls",
                "Count of calls to sqlite list_column_by_id_calls."
            )
            .unwrap(),
            columns_get_schema_calls: register_int_counter!(
                "sqlite_columns_get_schema_calls",
                "Count of calls to sqlite columns_get_schema_calls."
            )
            .unwrap(),
            put_object_calls: register_int_counter!(
                "sqlite_put_object_calls",
                "Count of calls to sqlite put_object_calls."
            )
            .unwrap(),
            get_object_calls: register_int_counter!(
                "sqlite_get_object_calls",
                "Count of calls to sqlite get_object_calls."
            )
            .unwrap(),
            run_query_calls: register_int_counter!(
                "sqlite_run_query_calls",
                "Count of calls to sqlite run_query_calls."
            )
            .unwrap(),
            execute_query_calls: register_int_counter!(
                "sqlite_execute_query_calls",
                "Count of calls to sqlite execute_query_calls."
            )
            .unwrap(),
            root_columns_list_by_id_calls: register_int_counter!(
                "sqlite_root_columns_list_by_id_calls",
                "Count of calls to sqlite root_columns_list_by_id_calls."
            )
            .unwrap(),
            new_root_columns_calls: register_int_counter!(
                "sqlite_new_root_columns_calls",
                "Count of calls to sqlite new_root_columns_calls."
            )
            .unwrap(),
            index_is_registered_calls: register_int_counter!(
                "sqlite_index_is_registered_calls",
                "Count of calls to sqlite index_is_registered_calls."
            )
            .unwrap(),
            register_index_calls: register_int_counter!(
                "sqlite_register_index_calls",
                "Count of calls to sqlite register_index_calls."
            )
            .unwrap(),
            registered_indices_calls: register_int_counter!(
                "sqlite_registered_indices_calls",
                "Count of calls to sqlite registered_indices_calls."
            )
            .unwrap(),
            index_asset_version_calls: register_int_counter!(
                "sqlite_index_asset_version_calls",
                "Count of calls to sqlite index_asset_version_calls."
            )
            .unwrap(),
            register_index_asset_calls: register_int_counter!(
                "sqlite_register_index_asset_calls",
                "Count of calls to sqlite register_index_asset_calls."
            )
            .unwrap(),
            latest_asset_for_index_calls: register_int_counter!(
                "sqlite_latest_asset_for_index_calls",
                "Count of calls to sqlite latest_asset_for_index_calls."
            )
            .unwrap(),
            latest_assets_for_index_calls: register_int_counter!(
                "sqlite_latest_assets_for_index_calls",
                "Count of calls to sqlite latest_assets_for_index_calls."
            )
            .unwrap(),
            asset_already_exists_calls: register_int_counter!(
                "sqlite_asset_already_exists_calls",
                "Count of calls to sqlite asset_already_exists_calls."
            )
            .unwrap(),
            index_id_for_calls: register_int_counter!(
                "sqlite_index_id_for_calls",
                "Count of calls to sqlite index_id_for_calls."
            )
            .unwrap(),
            start_transaction_calls: register_int_counter!(
                "sqlite_start_transaction_calls",
                "Count of calls to sqlite start_transaction_calls."
            )
            .unwrap(),
            commit_transaction_calls: register_int_counter!(
                "sqlite_commit_transaction_calls",
                "Count of calls to sqlite commit_transaction_calls."
            )
            .unwrap(),
            revert_transaction_calls: register_int_counter!(
                "sqlite_revert_transaction_calls",
                "Count of calls to sqlite revert_transaction_calls."
            )
            .unwrap(),
            run_migration_calls: register_int_counter!(
                "sqlite_run_migration_calls",
                "Count of calls to sqlite run_migration_calls."
            )
            .unwrap(),
            remove_index: register_int_counter!(
                "sqlite_remove_index",
                "Count of calls to sqlite remove_index."
            )
            .unwrap(),
        }
    }
}

use crate::core::Metric;
#[allow(unused)]
use prometheus::{self, register_int_counter, Counter, Encoder, IntCounter, TextEncoder};

#[derive(Debug, Clone)]
pub struct PostgreQueries {
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
    pub last_index_id_for_calls: IntCounter,
    pub penultimate_index_id_for_calls: IntCounter,
    pub penultimate_asset_for_index_calls: IntCounter,
    pub start_transaction_calls: IntCounter,
    pub commit_transaction_calls: IntCounter,
    pub revert_transaction_calls: IntCounter,
    pub run_migration_calls: IntCounter,
    pub remove_index: IntCounter,
    pub remove_asset_by_version_calls: IntCounter,
}

impl Metric for PostgreQueries {
    fn init() -> Self {
        Self {
            graph_root_latest_calls: register_int_counter!(
                "postgres_graph_root_latest_calls",
                "Count of calls to postgres graph_root_latest_calls."
            )
            .unwrap(),
            new_graph_root_calls: register_int_counter!(
                "postgres_new_graph_root_calls",
                "Count of calls to postgres new_graph_root_calls."
            )
            .unwrap(),
            type_id_list_by_name_calls: register_int_counter!(
                "postgres_type_id_list_by_name_calls",
                "Count of calls to postgres type_id_list_by_name_calls"
            )
            .unwrap(),
            type_id_latest_calls: register_int_counter!(
                "postgres_type_id_latest_calls",
                "Count of calls to postgres type_id_latest_calls."
            )
            .unwrap(),
            type_id_insert_calls: register_int_counter!(
                "postgres_type_id_insert_calls",
                "Count of calls to postgres type_id_insert_calls."
            )
            .unwrap(),
            schema_exists_calls: register_int_counter!(
                "postgres_schema_exists_calls",
                "Count of calls to postgres schema_exists_calls."
            )
            .unwrap(),
            new_column_insert_calls: register_int_counter!(
                "postgres_new_column_insert_calls",
                "Count of calls to postgres new_column_insert_calls."
            )
            .unwrap(),
            list_column_by_id_calls: register_int_counter!(
                "postgres_list_column_by_id_calls",
                "Count of calls to postgres list_column_by_id_calls."
            )
            .unwrap(),
            columns_get_schema_calls: register_int_counter!(
                "postgres_columns_get_schema_calls",
                "Count of calls to postgres columns_get_schema_calls."
            )
            .unwrap(),
            put_object_calls: register_int_counter!(
                "postgres_put_object_calls",
                "Count of calls to postgres put_object_calls."
            )
            .unwrap(),
            get_object_calls: register_int_counter!(
                "postgres_get_object_calls",
                "Count of calls to postgres get_object_calls."
            )
            .unwrap(),
            run_query_calls: register_int_counter!(
                "postgres_run_query_calls",
                "Count of calls to postgres run_query_calls."
            )
            .unwrap(),
            execute_query_calls: register_int_counter!(
                "postgres_execute_query_calls",
                "Count of calls to postgres execute_query_calls."
            )
            .unwrap(),
            root_columns_list_by_id_calls: register_int_counter!(
                "postgres_root_columns_list_by_id_calls",
                "Count of calls to postgres root_columns_list_by_id_calls."
            )
            .unwrap(),
            new_root_columns_calls: register_int_counter!(
                "postgres_new_root_columns_calls",
                "Count of calls to postgres new_root_columns_calls."
            )
            .unwrap(),
            index_is_registered_calls: register_int_counter!(
                "postgres_index_is_registered_calls",
                "Count of calls to postgres index_is_registered_calls."
            )
            .unwrap(),
            register_index_calls: register_int_counter!(
                "postgres_register_index_calls",
                "Count of calls to postgres register_index_calls."
            )
            .unwrap(),
            registered_indices_calls: register_int_counter!(
                "postgres_registered_indices_calls",
                "Count of calls to postgres registered_indices_calls."
            )
            .unwrap(),
            index_asset_version_calls: register_int_counter!(
                "postgres_index_asset_version_calls",
                "Count of calls to postgres index_asset_version_calls."
            )
            .unwrap(),
            register_index_asset_calls: register_int_counter!(
                "postgres_register_index_asset_calls",
                "Count of calls to postgres register_index_asset_calls."
            )
            .unwrap(),
            latest_asset_for_index_calls: register_int_counter!(
                "postgres_latest_asset_for_index_calls",
                "Count of calls to postgres latest_asset_for_index_calls."
            )
            .unwrap(),
            latest_assets_for_index_calls: register_int_counter!(
                "postgres_latest_assets_for_index_calls",
                "Count of calls to postgres latest_assets_for_index_calls."
            )
            .unwrap(),
            asset_already_exists_calls: register_int_counter!(
                "postgres_asset_already_exists_calls",
                "Count of calls to postgres asset_already_exists_calls."
            )
            .unwrap(),
            index_id_for_calls: register_int_counter!(
                "postgres_index_id_for_calls",
                "Count of calls to postgres index_id_for_calls."
            )
            .unwrap(),
            last_index_id_for_calls: register_int_counter!(
                "postgres_last_index_id_for_calls",
                "Count of calls to postgres last_index_id_for_calls."
            )
            .unwrap(),
            penultimate_index_id_for_calls: register_int_counter!(
                "postgres_penultimate_index_id_for_calls",
                "Count of calls to postgres penultimate_index_id_for_calls."
            )
            .unwrap(),
            penultimate_asset_for_index_calls: register_int_counter!(
                "postgres_penultimate_asset_id_for_calls",
                "Count of calls to postgres penultimate_index_id_for_calls."
            )
            .unwrap(),
            start_transaction_calls: register_int_counter!(
                "postgres_start_transaction_calls",
                "Count of calls to postgres start_transaction_calls."
            )
            .unwrap(),
            commit_transaction_calls: register_int_counter!(
                "postgres_commit_transaction_calls",
                "Count of calls to postgres commit_transaction_calls."
            )
            .unwrap(),
            revert_transaction_calls: register_int_counter!(
                "postgres_revert_transaction_calls",
                "Count of calls to postgres revert_transaction_calls."
            )
            .unwrap(),
            run_migration_calls: register_int_counter!(
                "postgres_run_migration_calls",
                "Count of calls to postgres run_migration_calls."
            )
            .unwrap(),
            remove_index: register_int_counter!(
                "postgres_remove_index",
                "Count of calls to postgres remove_index."
            )
            .unwrap(),
            remove_asset_by_version_calls: register_int_counter!(
                "postgres_remove_asset_by_version",
                "Count of calls to postgres remove_asset_by_version."
            )
            .unwrap(),
        }
    }
}

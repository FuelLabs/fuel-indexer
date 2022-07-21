table! {
    use diesel::sql_types::*;
    use crate::sql_types::*;

    graph_registry_columns (id) {
        id -> Int4,
        type_id -> Int8,
        column_position -> Int4,
        column_name -> Varchar,
        column_type -> Columntypename,
        nullable -> Bool,
        graphql_type -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::sql_types::*;

    graph_registry_graph_root (id) {
        id -> Int8,
        version -> Varchar,
        schema_name -> Varchar,
        query -> Varchar,
        schema -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::sql_types::*;

    graph_registry_root_columns (id) {
        id -> Int4,
        root_id -> Int8,
        column_name -> Varchar,
        graphql_type -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::sql_types::*;

    graph_registry_type_ids (id) {
        id -> Int8,
        schema_version -> Varchar,
        schema_name -> Varchar,
        graphql_name -> Varchar,
        table_name -> Varchar,
    }
}

joinable!(graph_registry_columns -> graph_registry_type_ids (type_id));
joinable!(graph_registry_root_columns -> graph_registry_graph_root (root_id));

allow_tables_to_appear_in_same_query!(
    graph_registry_columns,
    graph_registry_graph_root,
    graph_registry_root_columns,
    graph_registry_type_ids,
);

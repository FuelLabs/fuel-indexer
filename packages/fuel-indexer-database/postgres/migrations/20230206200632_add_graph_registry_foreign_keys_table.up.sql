create table if not exists graph_registry_foreign_keys (
    id bigserial primary key,
    schema_version varchar(512) not null,
    schema_name varchar(32) not null,
    schema_identifier varchar(255) not null,
    table_name varchar(32) not null,
    column_name varchar(32) not null,
    reference_table_name varchar(32) not null,
    reference_column_name varchar(32) not null,
    reference_column_type varchar(32) not null,
    db_type varchar(32) not null,
    namespace varchar(32) not null,
    on_delete varchar(32) not null,
    on_update varchar(32) not null
);
-- Add up migration script here
create table if not exists graph_registry_type_ids (
    id bigserial primary key,
    schema_version varchar(512) not null,
    schema_name varchar(32) not null,
    graphql_name varchar(32) not null,
    table_name varchar(32) not null
);

create table if not exists graph_registry_columns (
    id bigserial primary key,
    type_id bigserial not null,
    column_position integer not null,
    column_name varchar(32) not null,
    column_type varchar(32) not null,
    nullable boolean not null,
    graphql_type varchar not null,
    constraint fk_table_name
        foreign key(type_id)
            references graph_registry_type_ids(id)
);

CREATE TABLE IF NOT EXISTS graph_registry_graph_root (
    id bigserial primary key,
    version varchar not null,
    schema_name varchar not null,
    query varchar not null,
    schema varchar not null,
    UNIQUE(version, schema_name)
);

CREATE TABLE IF NOT EXISTS graph_registry_root_columns (
    id bigserial primary key,
    root_id bigserial not null,
    column_name varchar(32) not null,
    graphql_type varchar(32) not null,
    CONSTRAINT fk_root_id
        FOREIGN KEY(root_id)
            REFERENCES graph_registry_graph_root(id)
);

CREATE TABLE IF NOT EXISTS graph_registry_foreign_keys (
    id bigserial primary key,
    schema_version varchar(512) not null,
    schema_name varchar(32) not null,
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
alter table graph_registry_type_ids add column schema_identifier varchar(255) default 'unnamed';
alter table graph_registry_graph_root add column schema_identifier varchar(255) default 'unnamed';
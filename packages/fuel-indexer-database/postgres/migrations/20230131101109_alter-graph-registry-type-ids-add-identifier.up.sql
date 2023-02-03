alter table graph_registry_type_ids add column schema_identifier varchar(255) default 'unnamed';
alter table graph_registry_graph_root add column schema_identifier varchar(255) default 'unnamed';

alter table graph_registry_graph_root drop constraint graph_registry_graph_root_version_schema_name_key;
alter table graph_registry_graph_root add constraint graph_registry_graph_root_version_schema_name_schema_identifier_key unique(version, schema_name, schema_identifier);
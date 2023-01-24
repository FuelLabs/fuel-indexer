-- Add down migration script here
DROP TABLE graph_registry_type_ids cascade;
DROP TABLE graph_registry_columns cascade;
DROP TABLE graph_registry_graph_root;
DROP TABLE graph_registry_foreign_keys;
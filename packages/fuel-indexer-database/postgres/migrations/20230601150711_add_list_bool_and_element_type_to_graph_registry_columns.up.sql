alter table graph_registry_columns add column is_list_with_nullable_elements boolean null;
alter table graph_registry_columns add column inner_list_element_type varchar(256) null;
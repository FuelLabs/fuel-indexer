ALTER TABLE index_asset_registry_manifest ADD COLUMN version integer not null;
ALTER TABLE index_asset_registry_schema ADD COLUMN version integer not null;
ALTER TABLE index_asset_registry_wasm ADD COLUMN version integer not null;
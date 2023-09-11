-- Active: 1693382282533@@127.0.0.1@5432@postgres
ALTER TABLE index_asset_registry_manifest ADD COLUMN version integer not null;
ALTER TABLE index_asset_registry_schema ADD COLUMN version integer not null;
ALTER TABLE index_asset_registry_wasm ADD COLUMN version integer not null;
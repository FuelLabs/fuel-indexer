create table if not exists asset_registry (
   id bigserial primary key,
   namespace varchar(32) not null,
   identifier varchar(32) not null,
   wasm bytea not null,
   manifest bytea not null,
   schema bytea not null,
   UNIQUE(namespace, identifier)
);

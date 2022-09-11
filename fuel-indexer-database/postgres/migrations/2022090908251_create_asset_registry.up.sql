create table if not exists index_registry (
   id bigserial primary key,
   namespace varchar(32) not null,
   identifier varchar(32) not null,
   UNIQUE(namespace, identifier)
);

create table if not exists index_asset_registry_wasm (
   id bigserial primary key,
   index_id bigserial,
   version integer not null,
   bytes bytea not null,
    constraint fk_index_registry_id
        foreign key(index_id)
            references index_registry(id)
            on delete cascade
	        deferrable initially deferred
);


create table if not exists index_asset_registry_schema (
   id bigserial primary key,
   index_id bigserial,
   version integer not null,
   bytes bytea not null,
    constraint fk_index_registry_id
        foreign key(index_id)
            references index_registry(id)
            on delete cascade
	        deferrable initially deferred
);

create table if not exists index_asset_registry_manifest (
   id bigserial primary key,
   index_id bigserial,
   version integer not null,
   bytes bytea not null,
    constraint fk_index_registry_id
        foreign key(index_id)
            references index_registry(id)
            on delete cascade
	        deferrable initially deferred
);


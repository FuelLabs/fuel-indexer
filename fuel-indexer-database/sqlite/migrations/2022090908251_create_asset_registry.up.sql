create table if not exists index_registry (
   id integer primary key autoincrement,
   namespace varchar(32) not null,
   identifier varchar(32) not null,
   UNIQUE(namespace, identifier)
);

create table if not exists index_asset_registry_wasm (
   id integer primary key autoincrement,
   index_id integer not null,
   version integer not null,
   bytes blob not null,
    constraint fk_index_registry_id
        foreign key(index_id)
            references index_registry(id)
            on delete cascade
);


create table if not exists index_asset_registry_schema (
   id integer primary key autoincrement,
   index_id integer not null,
   version integer not null,
   bytes blob not null,
    constraint fk_index_registry_id
        foreign key(index_id)
            references index_registry(id)
            on delete cascade
);

create table if not exists index_asset_registry_manifest (
   id integer primary key autoincrement,
   index_id integer not null,
   version integer not null,
   bytes blob not null,
    constraint fk_index_registry_id
        foreign key(index_id)
            references index_registry(id)
            on delete cascade
);


create table auth (
    id bigserial primary key,
    indexer_id bigserial,
    token varchar(255),
    constraint fk_index_registry_id
    foreign key(indexer_id)
        references index_registry(id)
        on delete cascade
        deferrable initially deferred
);

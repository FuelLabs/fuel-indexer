create table auth (
    id primary key auto increment,
    index_id bigserial,
    token varchar(255),
    constraint fk_index_registry_id
    foreign key(index_id)
        references index_registry(id)
        on delete cascade
        deferrable initially deferred
);

create table nonce (
    id bigserial primary key,
    uid varchar(64) unique not null,
    expiry bigserial not null
);

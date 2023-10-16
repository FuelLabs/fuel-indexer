CREATE TABLE IF NOT EXISTS index_status (
   id bigserial primary key,
   namespace varchar(32) not null,
   identifier varchar(32) not null,
   status TEXT NOT NULL,
   status_message TEXT NOT NULL,
   UNIQUE(namespace, identifier)
);
CREATE TABLE IF NOT EXISTS index_status (
   indexer_id BIGSERIAL UNIQUE NOT NULL,
   status TEXT NOT NULL,
   status_message TEXT NOT NULL,
   FOREIGN KEY (indexer_id) REFERENCES index_registry (id)
);
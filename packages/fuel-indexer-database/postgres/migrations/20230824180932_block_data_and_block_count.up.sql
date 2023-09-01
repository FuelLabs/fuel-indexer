CREATE TABLE IF NOT EXISTS index_block_data (
    block_height INTEGER PRIMARY KEY NOT NULL,
    block_data BYTEA NOT NULL
);

CREATE OR REPLACE FUNCTION ensure_block_height_consecutive()
RETURNS TRIGGER AS $$
DECLARE
  block_height integer;
BEGIN
  EXECUTE format('SELECT MAX(block_height) FROM %I.%I', TG_TABLE_SCHEMA, TG_TABLE_NAME) INTO block_height;

  IF NEW.block_height IS NOT NULL AND block_height IS NOT NULL AND NEW.block_height != block_height + 1 THEN
    RAISE EXCEPTION '%.%: attempted to insert value with block_height = % while last indexed block_height = %. block_height values must be consecutive.', TG_TABLE_SCHEMA, TG_TABLE_NAME, NEW.block_height, block_height;
  END IF;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_ensure_block_height_consecutive
BEFORE INSERT OR UPDATE ON index_block_data
FOR EACH ROW
EXECUTE FUNCTION ensure_block_height_consecutive();
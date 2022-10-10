# Fuel Indexer

## Runtime Components

- database.rs - Database connections, schema management for metadata
  - Database: interfaces ffi functions with database queries for indexer WASM
  - SchemaManager: sets up, validates new graphql schemas
- executor.rs - wasm runtime environment
  - IndexEnv: holds references to objects that need to be available to ffi functions
  - IndexExecutor: load WASM, execute event triggers
- ffi.rs - functions callable from WASM, loading data structures to/from WASM
- manifest.rs - the yaml format for a graphql instance
  - namespace: The unique namespace this graphql schema lives in. This will correspond to the SQL database schema as well.
  - graphql_schema: file path for the graphql schema.
  - wasm_module: file path for the indexer WASM.
  - list of test events to run through the indexer
- schema.rs - SQL table schema builder

## Indexer Components

- fuel-indexer/lib - Crate with traits/types used in a WASM indexer
- fuel-indexer/derive - Derive macros to generate rust types from schema, and handler functions
- fuel-indexer/schema - Crate for common types between runtime and indexer, sql types, serialization, etc.

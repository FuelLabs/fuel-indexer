use fuel_indexer_schema::table::*;

#[cfg(test)]
mod tests {

    #[test]
    fn test_schema_builder_for_basic_postgres_schema_returns_proper_create_sql() {
        let graphql_schema: &str = r#"
        schema {
            query: QueryRoot
        }

        type QueryRoot {
            thing1: Thing1
            thing2: Thing2
        }

        type Thing1 {
            id: ID!
            account: Address!
        }

        type Thing2 {
            id: ID!
            account: Address!
            hash: Bytes32!
        }
    "#;

        let create_schema: &str = "CREATE SCHEMA IF NOT EXISTS test_namespace";
        let create_thing1_schmea: &str = concat!(
            "CREATE TABLE IF NOT EXISTS\n",
            " test_namespace.thing1 (\n",
            " id bigint primary key not null,\n",
            "account varchar(64) not null,\n",
            "object bytea not null",
            "\n)"
        );
        let create_thing2_schema: &str = concat!(
            "CREATE TABLE IF NOT EXISTS\n",
            " test_namespace.thing2 (\n",
            " id bigint primary key not null,\n",
            "account varchar(64) not null,\n",
            "hash varchar(64) not null,\n",
            "object bytea not null\n",
            ")"
        );

        let sb = SchemaBuilder::new("test_namespace", "a_version_string", DbType::Postgres);

        let SchemaBuilder { statements, .. } = sb.build(graphql_schema);

        assert_eq!(statements[0], create_schema);
        assert_eq!(statements[1], create_thing1_schmea);
        assert_eq!(statements[2], create_thing2_schema);
    }

    #[test]
    fn test_schema_builder_for_postgres_indices_returns_proper_create_sql() {
        let graphql_schema: &str = r#"
        schema {
            query: QueryRoot
        }

        type QueryRoot {
            thing1: Thing1
            thing2: Thing2
        }

        type Payer {
            id: ID!
            account: Address! @indexed
        }

        type Payee {
            id: ID!
            account: Address!
            hash: Bytes32! @indexed
        }
    "#;

        let sb = SchemaBuilder::new("namespace", "v1", DbType::Postgres);

        let SchemaBuilder { indices, .. } = sb.build(graphql_schema);

        assert_eq!(indices.len(), 2);
        assert_eq!(
            indices[0].create_statement(),
            "CREATE INDEX payer_account_idx ON namespace.payer USING btree (account);".to_string()
        );
        assert_eq!(
            indices[1].create_statement(),
            "CREATE INDEX payee_hash_idx ON namespace.payee USING btree (hash);".to_string()
        );
    }

    #[test]
    fn test_schema_builder_for_postgres_foreign_keys_returns_proper_create_sql() {
        let graphql_schema: &str = r#"
        schema {
            query: QueryRoot
        }

        type QueryRoot {
            borrower: Borrower
            lender: Lender
            auditor: Auditor
        }

        type Borrower {
            id: ID!
            account: Address! @indexed
        }

        type Lender {
            id: ID!
            account: Address!
            hash: Bytes32! @indexed
            borrower: Borrower!
        }

        type Auditor {
            id: ID!
            account: Address!
            hash: Bytes32! @indexed
            borrower: Borrower!
        }
    "#;

        let sb = SchemaBuilder::new("namespace", "v1", DbType::Postgres);

        let SchemaBuilder { foreign_keys, .. } = sb.build(graphql_schema);

        assert_eq!(foreign_keys.len(), 2);
        assert_eq!(foreign_keys[0].create_statement(), "ALTER TABLE namespace.lender ADD CONSTRAINT fk_borrower_id FOREIGN KEY (borrower) REFERENCES namespace.borrower(id) ON DELETE NO ACTION ON UPDATE NO ACTION INITIALLY DEFERRED;".to_string());
        assert_eq!(foreign_keys[1].create_statement(), "ALTER TABLE namespace.auditor ADD CONSTRAINT fk_borrower_id FOREIGN KEY (borrower) REFERENCES namespace.borrower(id) ON DELETE NO ACTION ON UPDATE NO ACTION INITIALLY DEFERRED;".to_string());
    }

    #[test]
    fn test_schema_builder_for_sqlite_indices_returns_proper_create_sql() {
        let graphql_schema: &str = r#"
        schema {
            query: QueryRoot
        }

        type QueryRoot {
            thing1: Thing1
            thing2: Thing2
        }

        type Payer {
            id: ID!
            account: Address! @indexed
        }

        type Payee {
            id: ID!
            account: Address!
            hash: Bytes32! @indexed
        }
    "#;

        let sb = SchemaBuilder::new("namespace", "v1", DbType::Sqlite);

        let SchemaBuilder { indices, .. } = sb.build(graphql_schema);

        assert_eq!(indices.len(), 2);
        assert_eq!(
            indices[0].create_statement(),
            "CREATE INDEX payer_account_idx ON payer(account);".to_string()
        );
        assert_eq!(
            indices[1].create_statement(),
            "CREATE INDEX payee_hash_idx ON payee(hash);".to_string()
        );
    }

    #[test]
    fn test_schema_builder_for_sqlite_foreign_keys_returns_proper_create_sql() {
        let graphql_schema: &str = r#"
        schema {
            query: QueryRoot
        }

        type QueryRoot {
            borrower: Borrower
            lender: Lender
            auditor: Auditor
        }

        type Borrower {
            id: ID!
            account: Address! @indexed
        }

        type Lender {
            id: ID!
            account: Address!
            hash: Bytes32! @indexed
            borrower: Borrower!
        }

        type Auditor {
            id: ID!
            account: Address!
            hash: Bytes32! @indexed
            borrower: Borrower!
        }
    "#;

        let sb = SchemaBuilder::new("namespace", "v1", DbType::Sqlite);

        let SchemaBuilder { foreign_keys, .. } = sb.build(graphql_schema);

        assert_eq!(foreign_keys.len(), 2);
        assert_eq!(foreign_keys[0].create_statement(), "ALTER TABLE lender DROP COLUMN borrower; ALTER TABLE lender ADD COLUMN borrower BIGINT REFERENCES borrower(id);");
        assert_eq!(foreign_keys[1].create_statement(), "ALTER TABLE auditor DROP COLUMN borrower; ALTER TABLE auditor ADD COLUMN borrower BIGINT REFERENCES borrower(id);");
    }
}

use crate::utils::{
    build_schema_fields_and_types_map, build_schema_objects_set, field_type_table_name,
    get_index_directive, get_join_directive_info, get_unique_directive,
    normalize_field_type_name, BASE_SCHEMA,
};
use fuel_indexer_database::{
    queries,
    types::{directives, *},
    DbType, IndexerConnection, IndexerConnectionPool,
};
use fuel_indexer_types::type_id;
use graphql_parser::parse_schema;
use graphql_parser::schema::{
    Definition, Field, ObjectType, SchemaDefinition, Type, TypeDefinition,
};
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct SchemaBuilder {
    db_type: DbType,
    statements: Vec<String>,
    type_ids: Vec<TypeId>,
    columns: Vec<NewColumn>,
    foreign_keys: Vec<ForeignKey>,
    indices: Vec<ColumnIndex>,
    namespace: String,
    version: String,
    schema: String,
    types: HashSet<String>,
    fields: HashMap<String, HashMap<String, String>>,
    query: String,
    query_fields: HashMap<String, HashMap<String, String>>,
    primitives: HashSet<String>,
}

impl SchemaBuilder {
    pub fn new(namespace: &str, version: &str, db_type: DbType) -> SchemaBuilder {
        let base_ast = match parse_schema::<String>(BASE_SCHEMA) {
            Ok(ast) => ast,
            Err(e) => {
                panic!("Error parsing graphql schema {e:?}",)
            }
        };
        let (primitives, _) = build_schema_objects_set(&base_ast);

        SchemaBuilder {
            db_type,
            namespace: namespace.to_string(),
            version: version.to_string(),
            primitives,
            ..Default::default()
        }
    }

    pub fn build(mut self, schema: &str) -> Self {
        if DbType::Postgres == self.db_type {
            let create = format!("CREATE SCHEMA IF NOT EXISTS {}", self.namespace);
            self.statements.push(create);
        }

        let ast = match parse_schema::<String>(schema) {
            Ok(ast) => ast,
            Err(e) => panic!("Error parsing graphql schema {e:?}",),
        };

        let query = ast
            .definitions
            .iter()
            .filter_map(|s| {
                if let Definition::SchemaDefinition(def) = s {
                    let SchemaDefinition { query, .. } = def;
                    query.as_ref()
                } else {
                    None
                }
            })
            .next();

        // TODO: Add error enum here
        let query = query.cloned().expect("TODO: this needs to be error type");

        let types_map = build_schema_fields_and_types_map(&ast);

        for def in ast.definitions.iter() {
            if let Definition::TypeDefinition(typ) = def {
                self.generate_table_sql(&query, typ, &types_map);
            }
        }

        self.query = query;
        self.schema = schema.to_string();

        self
    }

    pub async fn commit_metadata(
        self,
        conn: &mut IndexerConnection,
    ) -> sqlx::Result<Schema> {
        #[allow(unused_variables)]
        let SchemaBuilder {
            version,
            statements,
            type_ids,
            columns,
            foreign_keys,
            indices,
            namespace,
            types,
            fields,
            query,
            query_fields,
            schema,
            db_type,
            ..
        } = self;

        let new_root = NewGraphRoot {
            version: version.clone(),
            schema_name: namespace.clone(),
            query: query.clone(),
            schema,
        };
        queries::new_graph_root(conn, new_root).await?;

        let latest = queries::graph_root_latest(conn, &namespace).await?;

        let field_defs = query_fields.get(&query).expect("No query root.");

        let cols: Vec<_> = field_defs
            .iter()
            .map(|(key, val)| NewRootColumns {
                root_id: latest.id,
                column_name: key.to_string(),
                graphql_type: val.to_string(),
            })
            .collect();

        queries::new_root_columns(conn, cols).await?;

        for query in statements {
            queries::execute_query(conn, query).await?;
        }

        for fk in foreign_keys {
            queries::execute_query(conn, fk.create_statement()).await?;
        }

        for idx in indices {
            queries::execute_query(conn, idx.create_statement()).await?;
        }

        queries::type_id_insert(conn, type_ids).await?;
        queries::new_column_insert(conn, columns).await?;

        Ok(Schema {
            version,
            namespace,
            query,
            types,
            fields,
        })
    }

    fn process_type(&self, field_type: &Type<String>) -> (ColumnType, bool) {
        match field_type {
            Type::NamedType(t) => {
                if !self.primitives.contains(t.as_str()) {
                    return (ColumnType::ForeignKey, true);
                }
                (ColumnType::from(t.as_str()), true)
            }
            Type::ListType(_) => panic!("List types not supported yet."),
            Type::NonNullType(t) => {
                let (typ, _) = self.process_type(t);
                (typ, false)
            }
        }
    }

    fn generate_columns<'a>(
        &mut self,
        obj: &ObjectType<'a, String>,
        type_id: i64,
        fields: &[Field<'a, String>],
        table_name: &str,
        types_map: &HashMap<String, String>,
    ) -> String {
        let mut fragments = Vec::new();

        for (pos, field) in fields.iter().enumerate() {
            let (typ, nullable) = self.process_type(&field.field_type);

            let directives::Unique(unique) = get_unique_directive(field);

            if typ == ColumnType::ForeignKey {
                let directives::Join {
                    reference_field_name,
                    field_type_name,
                    reference_field_type_name,
                    ..
                } = get_join_directive_info(field, obj, types_map);

                let fk = ForeignKey::new(
                    self.db_type.clone(),
                    self.namespace.clone(),
                    table_name.to_string(),
                    field.name.clone(),
                    field_type_table_name(field),
                    reference_field_name.clone(),
                    reference_field_type_name.to_owned(),
                );

                let column = NewColumn {
                    type_id,
                    column_position: pos as i32,
                    column_name: field.name.to_string(),
                    column_type: reference_field_type_name.to_owned(),
                    graphql_type: field_type_name,
                    nullable,
                    unique,
                };

                fragments.push(column.sql_fragment());
                self.columns.push(column);
                self.foreign_keys.push(fk);

                continue;
            }

            let column = NewColumn {
                type_id,
                column_position: pos as i32,
                column_name: field.name.to_string(),
                column_type: typ.to_string(),
                graphql_type: field.field_type.to_string(),
                nullable,
                unique,
            };

            if let Some(directives::Index {
                column_name,
                method,
            }) = get_index_directive(field)
            {
                self.indices.push(ColumnIndex {
                    db_type: self.db_type.clone(),
                    table_name: table_name.to_string(),
                    namespace: self.namespace.to_string(),
                    method,
                    unique,
                    column_name,
                });
            }

            fragments.push(column.sql_fragment());
            self.columns.push(column);
        }

        let object_column = NewColumn {
            type_id,
            column_position: fragments.len() as i32,
            // FIXME: Magic strings here
            column_name: "object".to_string(),
            column_type: "Object".to_string(),
            graphql_type: "__".into(),
            nullable: false,
            unique: false,
        };

        fragments.push(object_column.sql_fragment());
        self.columns.push(object_column);

        fragments.join(",\n")
    }

    fn generate_table_sql(
        &mut self,
        root: &str,
        typ: &TypeDefinition<String>,
        types_map: &HashMap<String, String>,
    ) {
        fn map_fields(fields: &[Field<String>]) -> HashMap<String, String> {
            fields
                .iter()
                .map(|f| (f.name.to_string(), f.field_type.to_string()))
                .collect()
        }

        match typ {
            TypeDefinition::Object(o) => {
                self.types.insert(o.name.to_string());
                self.fields
                    .insert(o.name.to_string(), map_fields(&o.fields));

                if o.name == root {
                    self.query_fields
                        .insert(root.to_string(), map_fields(&o.fields));
                    return;
                }

                let table_name = o.name.to_lowercase();
                let type_id = type_id(&self.namespace, &o.name);
                let columns =
                    self.generate_columns(o, type_id, &o.fields, &table_name, types_map);

                let sql_table = self.db_type.table_name(&self.namespace, &table_name);

                let create =
                    format!("CREATE TABLE IF NOT EXISTS\n {sql_table} (\n {columns}\n)",);

                self.statements.push(create);
                self.type_ids.push(TypeId {
                    id: type_id,
                    schema_version: self.version.to_string(),
                    schema_name: self.namespace.to_string(),
                    graphql_name: o.name.to_string(),
                    table_name,
                });
            }
            o => panic!("Got a non-object type: '{o:?}'"),
        }
    }
}
#[derive(Debug)]
pub struct Schema {
    pub version: String,
    pub namespace: String,
    pub query: String,
    pub types: HashSet<String>,
    pub fields: HashMap<String, HashMap<String, String>>,
}

impl Schema {
    pub async fn load_from_db(
        pool: &IndexerConnectionPool,
        name: &str,
    ) -> sqlx::Result<Self> {
        let mut conn = pool.acquire().await?;
        let root = queries::graph_root_latest(&mut conn, name).await?;
        let root_cols = queries::root_columns_list_by_id(&mut conn, root.id).await?;
        let typeids =
            queries::type_id_list_by_name(&mut conn, &root.schema_name, &root.version)
                .await?;

        let mut types = HashSet::new();
        let mut fields = HashMap::new();

        types.insert(root.query.clone());
        fields.insert(
            root.query.clone(),
            root_cols
                .into_iter()
                .map(|c| (c.column_name, c.graphql_type))
                .collect(),
        );
        for tid in typeids {
            types.insert(tid.graphql_name.clone());

            let columns = queries::list_column_by_id(&mut conn, tid.id).await?;
            fields.insert(
                tid.graphql_name,
                columns
                    .into_iter()
                    .map(|c| (c.column_name, c.graphql_type))
                    .collect(),
            );
        }

        Ok(Schema {
            version: root.version,
            namespace: root.schema_name,
            query: root.query,
            types,
            fields,
        })
    }

    pub fn check_type(&self, type_name: &str) -> bool {
        self.types.contains(type_name)
    }

    pub fn field_type(&self, cond: &str, name: &str) -> Option<&String> {
        match self.fields.get(cond) {
            Some(fieldset) => fieldset.get(name),
            _ => {
                let tablename = normalize_field_type_name(cond);
                match self.fields.get(&tablename) {
                    Some(fieldset) => fieldset.get(name),
                    _ => None,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let sb =
            SchemaBuilder::new("test_namespace", "a_version_string", DbType::Postgres);

        let SchemaBuilder { statements, .. } = sb.build(graphql_schema);

        assert_eq!(statements[0], create_schema);
        assert_eq!(statements[1], create_thing1_schmea);
        assert_eq!(statements[2], create_thing2_schema);
    }

    #[test]
    fn test_schema_builder_for_basic_postgres_schema_with_optional_types_returns_proper_create_sql(
    ) {
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
            account: Address
        }

        type Thing2 {
            id: ID!
            account: Address
            hash: Bytes32
        }
    "#;

        let create_schema: &str = "CREATE SCHEMA IF NOT EXISTS test_namespace";
        let create_thing1_schmea: &str = concat!(
            "CREATE TABLE IF NOT EXISTS\n",
            " test_namespace.thing1 (\n",
            " id bigint primary key not null,\n",
            "account varchar(64),\n",
            "object bytea not null",
            "\n)"
        );
        let create_thing2_schema: &str = concat!(
            "CREATE TABLE IF NOT EXISTS\n",
            " test_namespace.thing2 (\n",
            " id bigint primary key not null,\n",
            "account varchar(64),\n",
            "hash varchar(64),\n",
            "object bytea not null\n",
            ")"
        );

        let sb =
            SchemaBuilder::new("test_namespace", "a_version_string", DbType::Postgres);

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
            "CREATE INDEX payer_account_idx ON namespace.payer USING btree (account);"
                .to_string()
        );
        assert_eq!(
            indices[1].create_statement(),
            "CREATE INDEX payee_hash_idx ON namespace.payee USING btree (hash);"
                .to_string()
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
        assert_eq!(foreign_keys[0].create_statement(), "ALTER TABLE namespace.lender ADD CONSTRAINT fk_lender_borrower__borrower_id FOREIGN KEY (borrower) REFERENCES namespace.borrower(id) ON DELETE NO ACTION ON UPDATE NO ACTION INITIALLY DEFERRED;".to_string());
        assert_eq!(foreign_keys[1].create_statement(), "ALTER TABLE namespace.auditor ADD CONSTRAINT fk_auditor_borrower__borrower_id FOREIGN KEY (borrower) REFERENCES namespace.borrower(id) ON DELETE NO ACTION ON UPDATE NO ACTION INITIALLY DEFERRED;".to_string());
    }

    #[test]
    fn test_schema_builder_for_postgres_foreign_keys_with_directive_returns_proper_create_sql(
    ) {
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
            account: Address! @indexed
        }

        type Lender {
            id: ID!
            borrower: Borrower! @join(on:account)
        }

        type Auditor {
            id: ID!
            account: Address!
            hash: Bytes32! @indexed
            borrower: Borrower! @join(on:account)
        }
    "#;

        let sb = SchemaBuilder::new("namespace", "v1", DbType::Postgres);

        let SchemaBuilder { foreign_keys, .. } = sb.build(graphql_schema);

        assert_eq!(foreign_keys.len(), 2);
        assert_eq!(foreign_keys[0].create_statement(), "ALTER TABLE namespace.lender ADD CONSTRAINT fk_lender_borrower__borrower_account FOREIGN KEY (borrower) REFERENCES namespace.borrower(account) ON DELETE NO ACTION ON UPDATE NO ACTION INITIALLY DEFERRED;".to_string());
        assert_eq!(foreign_keys[1].create_statement(), "ALTER TABLE namespace.auditor ADD CONSTRAINT fk_auditor_borrower__borrower_account FOREIGN KEY (borrower) REFERENCES namespace.borrower(account) ON DELETE NO ACTION ON UPDATE NO ACTION INITIALLY DEFERRED;".to_string());
    }

    #[test]
    fn test_schema_builder_for_postgres_creates_fk_with_proper_column_names() {
        let graphql_schema: &str = r#"
        schema {
            query: QueryRoot
        }

        type QueryRoot {
            account: Account
            message: Message
        }

        type Account {
            id: ID!
            account: Address! @indexed
        }

        type Message {
            id: ID!
            sender: Account!
            receiver: Account!
        }
    "#;

        let sb = SchemaBuilder::new("namespace", "v1", DbType::Postgres);

        let SchemaBuilder { foreign_keys, .. } = sb.build(graphql_schema);

        assert_eq!(foreign_keys.len(), 2);
        assert_eq!(foreign_keys[0].create_statement(), "ALTER TABLE namespace.message ADD CONSTRAINT fk_message_sender__account_id FOREIGN KEY (sender) REFERENCES namespace.account(id) ON DELETE NO ACTION ON UPDATE NO ACTION INITIALLY DEFERRED;".to_string());
        assert_eq!(foreign_keys[1].create_statement(), "ALTER TABLE namespace.message ADD CONSTRAINT fk_message_receiver__account_id FOREIGN KEY (receiver) REFERENCES namespace.account(id) ON DELETE NO ACTION ON UPDATE NO ACTION INITIALLY DEFERRED;".to_string());
    }
}

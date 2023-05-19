use crate::{
    db::{IndexerSchemaDbError, IndexerSchemaDbResult},
    parser::ParsedGraphQLSchema,
    utils::*,
    QUERY_ROOT,
};
use async_graphql_parser::{
    parse_schema,
    types::{
        BaseType, FieldDefinition, ServiceDocument, Type, TypeDefinition, TypeKind,
        TypeSystemDefinition,
    },
};
use fuel_indexer_database::{
    queries, types::*, DbType, IndexerConnection, IndexerConnectionPool,
};
use fuel_indexer_types::type_id;
use std::collections::{HashMap, HashSet};

type ForeignKeyMap = HashMap<String, HashMap<String, (String, String)>>;
type ServiceDocumentBundle = (ServiceDocument, HashSet<String>, HashMap<String, String>);

#[derive(Default)]
pub struct SchemaBuilder {
    db_type: DbType,
    statements: Vec<String>,
    type_ids: Vec<TypeId>,
    columns: Vec<NewColumn>,
    foreign_keys: Vec<ForeignKey>,
    indices: Vec<ColumnIndex>,
    namespace: String,
    identifier: String,
    version: String,
    schema: String,
    types: HashSet<String>,
    /// Schema field mapping is namespaced by type name
    fields: HashMap<String, HashMap<String, String>>,
    primitives: HashSet<String>,
    parsed_schema: ParsedGraphQLSchema,
    is_native: bool,
}

impl SchemaBuilder {
    pub fn new(
        namespace: &str,
        identifier: &str,
        version: &str,
        db_type: DbType,
        is_native: bool,
    ) -> IndexerSchemaDbResult<SchemaBuilder> {
        let ast = parse_schema(BASE_SCHEMA).map_err(IndexerSchemaDbError::ParseError)?;

        let parsed_schema =
            ParsedGraphQLSchema::new(&namespace, &identifier, is_native, &ast)?;

        Ok(SchemaBuilder {
            db_type,
            namespace: namespace.to_string(),
            identifier: identifier.to_string(),
            version: version.to_string(),
            primitives: parsed_schema.scalar_names.clone(),
            parsed_schema,
            is_native,
            ..Default::default()
        })
    }

    /// Generate table SQL for each object in the given schema.
    pub fn build(mut self, schema: &str) -> IndexerSchemaDbResult<Self> {
        if DbType::Postgres == self.db_type {
            let create = format!(
                "CREATE SCHEMA IF NOT EXISTS {}_{}",
                self.namespace, self.identifier
            );
            self.statements.push(create);
        }

        let ast = parse_schema(schema).map_err(IndexerSchemaDbError::ParseError)?;

        let parsed_schema = ParsedGraphQLSchema::new(
            &self.namespace,
            &self.identifier,
            self.is_native,
            &ast,
        )?;

        for def in ast.definitions.iter() {
            if let TypeSystemDefinition::Type(typ) = def {
                self.generate_table_sql(&typ.node, &parsed_schema.field_type_mappings)
            }
        }
        self.schema = schema.to_string();
        self.parsed_schema = parsed_schema;

        Ok(self)
    }

    /// Commit all SQL metadata to the database.
    pub async fn commit_metadata(
        self,
        conn: &mut IndexerConnection,
    ) -> IndexerSchemaDbResult<Schema> {
        let SchemaBuilder {
            version,
            statements,
            type_ids,
            columns,
            foreign_keys,
            indices,
            namespace,
            identifier,
            types,
            fields,
            schema,
            ..
        } = self;

        let new_root = NewGraphRoot {
            version: version.clone(),
            schema_name: namespace.clone(),
            schema_identifier: identifier.clone(),
            schema: schema.clone(),
        };
        queries::new_graph_root(conn, new_root).await?;

        let latest = queries::graph_root_latest(conn, &namespace, &identifier).await?;

        let type_names = fields.keys();

        let cols: Vec<_> = type_names
            .map(|t| NewRootColumns {
                root_id: latest.id,
                column_name: t.to_string(),
                graphql_type: t.to_string(),
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

        let mut schema = Schema {
            version,
            namespace,
            identifier,
            types,
            fields,
            foreign_keys: HashMap::new(),
        };
        schema.register_queryroot_fields();

        Ok(schema)
    }

    fn process_type(&self, field_type: &Type) -> (ColumnType, bool) {
        match &field_type.base {
            BaseType::Named(t) => {
                // THIS NEEDS TO HANDLE ENUMS
                if !self.primitives.contains(t.as_str()) {
                    return (ColumnType::ForeignKey, true);
                }
                (ColumnType::from(t.as_str()), field_type.nullable)
            }
            BaseType::List(_) => panic!("List types not supported yet."),
        }
    }

    fn generate_columns(
        &mut self,
        type_name: &String,
        type_id: i64,
        fields: &[FieldDefinition],
        table_name: &str,
        types_map: &HashMap<String, String>,
    ) -> String {
        let mut fragments = Vec::new();

        for (pos, field) in fields.iter().enumerate() {
            let (typ, nullable) = self.process_type(&field.ty.node);

            let directives::Unique(unique) = get_unique_directive(field);

            if typ == ColumnType::ForeignKey {
                let directives::Join {
                    reference_field_name,
                    field_type_name,
                    reference_field_type_name,
                    ..
                } = get_join_directive_info(field, type_name, types_map);

                let fk = ForeignKey::new(
                    self.db_type.clone(),
                    self.namespace(),
                    table_name.to_string(),
                    field.name.to_string(),
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
                graphql_type: field.ty.to_string(),
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
                    namespace: self.namespace(),
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

    fn namespace(&self) -> String {
        format!("{}_{}", self.namespace, self.identifier)
    }

    fn generate_table_sql(
        &mut self,
        typ: &TypeDefinition,
        types_map: &HashMap<String, String>,
    ) {
        match &typ.kind {
            TypeKind::Scalar => {}
            TypeKind::Enum(_e) => {}
            TypeKind::Object(o) => {
                self.types.insert(typ.name.to_string());

                let field_defs: &[FieldDefinition] = &o
                    .fields
                    .iter()
                    .map(|f| f.node.clone())
                    .collect::<Vec<FieldDefinition>>()[..];

                self.fields.insert(
                    typ.name.to_string(),
                    field_defs
                        .iter()
                        .map(|f| (f.name.to_string(), f.ty.to_string()))
                        .collect(),
                );

                let table_name = typ.name.to_string().to_lowercase();
                let type_id = type_id(&self.namespace(), &typ.name.to_string());
                let columns = self.generate_columns(
                    &typ.name.to_string(),
                    type_id,
                    field_defs,
                    &table_name,
                    types_map,
                );

                let sql_table = self.db_type.table_name(&self.namespace(), &table_name);

                let create =
                    format!("CREATE TABLE IF NOT EXISTS\n {sql_table} (\n {columns}\n)",);

                self.statements.push(create);
                self.type_ids.push(TypeId {
                    id: type_id,
                    schema_version: self.version.to_string(),
                    schema_name: self.namespace.to_string(),
                    schema_identifier: self.identifier.to_string(),
                    graphql_name: typ.name.to_string(),
                    table_name,
                });
            }
            // TODO: Don't panic, return Err
            other_type => panic!("Got a non-object type: '{other_type:?}'"),
        }
    }
}
#[derive(Debug, Clone)]
pub struct Schema {
    pub version: String,
    pub namespace: String,
    pub identifier: String,
    pub types: HashSet<String>,
    /// Schema field mapping is namespaced by type name
    pub fields: HashMap<String, HashMap<String, String>>,
    pub foreign_keys: HashMap<String, HashMap<String, (String, String)>>,
}

impl Schema {
    pub async fn load_from_db(
        pool: &IndexerConnectionPool,
        namespace: &str,
        identifier: &str,
    ) -> IndexerSchemaDbResult<Self> {
        let mut conn = pool.acquire().await?;
        let root = queries::graph_root_latest(&mut conn, namespace, identifier).await?;
        let typeids = queries::type_id_list_by_name(
            &mut conn,
            &root.schema_name,
            &root.version,
            identifier,
        )
        .await?;

        let mut types = HashSet::new();
        let mut fields = HashMap::new();

        for tyid in &typeids {
            types.insert(tyid.graphql_name.clone());

            let columns = queries::list_column_by_id(&mut conn, tyid.id).await?;
            fields.insert(
                tyid.graphql_name.to_owned(),
                columns
                    .into_iter()
                    .map(|c| (c.column_name, c.graphql_type))
                    .collect(),
            );
        }

        let foreign_keys = get_foreign_keys(&root.schema)?;

        let mut schema = Schema {
            version: root.version,
            namespace: root.schema_name,
            identifier: root.schema_identifier,
            types,
            fields,
            foreign_keys,
        };

        schema.register_queryroot_fields();

        Ok(schema)
    }

    /// Return the field type for a given field name on a given type name.
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

    /// Ensure the given type is included in this `Schema`'s types
    pub fn check_type(&self, type_name: &str) -> bool {
        self.types.contains(type_name)
    }

    // **** HACK ****

    // Below we manually add a `QueryRoot` type, with its corresponding field types
    // data being each `Object` defined in the schema.

    // We need this because at the moment our GraphQL query parsing is tightly-coupled
    // to our old way of resolving GraphQL types (which was using a `QueryType` object
    // defined in a `TypeSystemDefinition::Schema`)
    pub fn register_queryroot_fields(&mut self) {
        self.fields.insert(
            QUERY_ROOT.to_string(),
            self.fields
                .keys()
                .map(|k| (k.to_lowercase(), k.clone()))
                .collect::<HashMap<String, String>>(),
        );
    }
}

fn get_foreign_keys(schema: &str) -> IndexerSchemaDbResult<ForeignKeyMap> {
    let (ast, primitives, types_map) = parse_schema_for_ast_data(schema)?;
    let mut fks: ForeignKeyMap = HashMap::new();

    for def in ast.definitions.iter() {
        if let TypeSystemDefinition::Type(t) = def {
            if let TypeKind::Object(o) = &t.node.kind {
                // TODO: Add more ignorable types as needed - and use lazy_static!
                if t.node.name.to_string().to_lowercase() == *"queryroot" {
                    continue;
                }
                for field in o.fields.iter() {
                    let col_type = get_column_type(&field.node.ty.node, &primitives)?;
                    #[allow(clippy::single_match)]
                    match col_type {
                        ColumnType::ForeignKey => {
                            let directives::Join {
                                reference_field_name,
                                ..
                            } = get_join_directive_info(
                                &field.node,
                                &t.node.name.to_string(),
                                &types_map,
                            );

                            let fk = fks.get_mut(&t.node.name.to_string().to_lowercase());
                            match fk {
                                Some(fks_for_field) => {
                                    fks_for_field.insert(
                                        field.node.name.to_string(),
                                        (
                                            field_type_table_name(&field.node),
                                            reference_field_name.clone(),
                                        ),
                                    );
                                }
                                None => {
                                    let fks_for_field = HashMap::from([(
                                        field.node.name.to_string(),
                                        (
                                            field_type_table_name(&field.node),
                                            reference_field_name.clone(),
                                        ),
                                    )]);
                                    fks.insert(
                                        t.node.name.to_string().to_lowercase(),
                                        fks_for_field,
                                    );
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(fks)
}

fn parse_schema_for_ast_data(
    schema: &str,
) -> IndexerSchemaDbResult<ServiceDocumentBundle> {
    let base_ast = parse_schema(BASE_SCHEMA).map_err(IndexerSchemaDbError::ParseError)?;
    let ast = parse_schema(schema).map_err(IndexerSchemaDbError::ParseError)?;
    let (primitives, _) = build_schema_types_set(&base_ast);

    let types_map = build_schema_fields_and_types_map(&ast)?;

    Ok((ast, primitives, types_map))
}

fn get_column_type(
    field_type: &Type,
    primitives: &HashSet<String>,
) -> IndexerSchemaDbResult<ColumnType> {
    match &field_type.base {
        BaseType::Named(t) => {
            if !primitives.contains(t.as_str()) {
                return Ok(ColumnType::ForeignKey);
            }
            Ok(ColumnType::from(t.as_str()))
        }
        BaseType::List(_) => Err(IndexerSchemaDbError::ListTypesUnsupported),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_builder_for_basic_postgres_schema_returns_proper_create_sql() {
        let graphql_schema: &str = r#"
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

        let create_schema: &str = "CREATE SCHEMA IF NOT EXISTS test_namespace_index1";
        let create_thing1_schmea: &str = concat!(
            "CREATE TABLE IF NOT EXISTS\n",
            " test_namespace_index1.thing1 (\n",
            " id numeric(20, 0) primary key not null,\n",
            "account varchar(64) not null,\n",
            "object bytea not null",
            "\n)"
        );
        let create_thing2_schema: &str = concat!(
            "CREATE TABLE IF NOT EXISTS\n",
            " test_namespace_index1.thing2 (\n",
            " id numeric(20, 0) primary key not null,\n",
            "account varchar(64) not null,\n",
            "hash varchar(64) not null,\n",
            "object bytea not null\n",
            ")"
        );

        let sb = SchemaBuilder::new(
            "test_namespace",
            "index1",
            "a_version_string",
            DbType::Postgres,
        );

        let SchemaBuilder { statements, .. } = sb.unwrap().build(graphql_schema).unwrap();

        assert_eq!(statements[0], create_schema);
        assert_eq!(statements[1], create_thing1_schmea);
        assert_eq!(statements[2], create_thing2_schema);
    }

    #[test]
    fn test_schema_builder_for_basic_postgres_schema_with_optional_types_returns_proper_create_sql(
    ) {
        let graphql_schema: &str = r#"
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

        let create_schema: &str = "CREATE SCHEMA IF NOT EXISTS test_namespace_index1";
        let create_thing1_schmea: &str = concat!(
            "CREATE TABLE IF NOT EXISTS\n",
            " test_namespace_index1.thing1 (\n",
            " id numeric(20, 0) primary key not null,\n",
            "account varchar(64),\n",
            "object bytea not null",
            "\n)"
        );
        let create_thing2_schema: &str = concat!(
            "CREATE TABLE IF NOT EXISTS\n",
            " test_namespace_index1.thing2 (\n",
            " id numeric(20, 0) primary key not null,\n",
            "account varchar(64),\n",
            "hash varchar(64),\n",
            "object bytea not null\n",
            ")"
        );

        let sb = SchemaBuilder::new(
            "test_namespace",
            "index1",
            "a_version_string",
            DbType::Postgres,
        );

        let SchemaBuilder { statements, .. } = sb.unwrap().build(graphql_schema).unwrap();

        assert_eq!(statements[0], create_schema);
        assert_eq!(statements[1], create_thing1_schmea);
        assert_eq!(statements[2], create_thing2_schema);
    }

    #[test]
    fn test_schema_builder_for_postgres_indices_returns_proper_create_sql() {
        let graphql_schema: &str = r#"
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

        let sb = SchemaBuilder::new("namespace", "index1", "v1", DbType::Postgres);

        let SchemaBuilder { indices, .. } = sb.unwrap().build(graphql_schema).unwrap();

        assert_eq!(indices.len(), 2);
        assert_eq!(
            indices[0].create_statement(),
            "CREATE INDEX payer_account_idx ON namespace_index1.payer USING btree (account);"
                .to_string()
        );
        assert_eq!(
            indices[1].create_statement(),
            "CREATE INDEX payee_hash_idx ON namespace_index1.payee USING btree (hash);"
                .to_string()
        );
    }

    #[test]
    fn test_schema_builder_for_postgres_foreign_keys_returns_proper_create_sql() {
        let graphql_schema: &str = r#"
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

        let sb = SchemaBuilder::new("namespace", "index1", "v1", DbType::Postgres);

        let SchemaBuilder { foreign_keys, .. } =
            sb.unwrap().build(graphql_schema).unwrap();

        assert_eq!(foreign_keys.len(), 2);
        assert_eq!(foreign_keys[0].create_statement(), "ALTER TABLE namespace_index1.lender ADD CONSTRAINT fk_lender_borrower__borrower_id FOREIGN KEY (borrower) REFERENCES namespace_index1.borrower(id) ON DELETE NO ACTION ON UPDATE NO ACTION INITIALLY DEFERRED;".to_string());
        assert_eq!(foreign_keys[1].create_statement(), "ALTER TABLE namespace_index1.auditor ADD CONSTRAINT fk_auditor_borrower__borrower_id FOREIGN KEY (borrower) REFERENCES namespace_index1.borrower(id) ON DELETE NO ACTION ON UPDATE NO ACTION INITIALLY DEFERRED;".to_string());
    }

    #[test]
    fn test_schema_builder_for_postgres_foreign_keys_with_directive_returns_proper_create_sql(
    ) {
        let graphql_schema: &str = r#"
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

        let sb = SchemaBuilder::new("namespace", "index1", "v1", DbType::Postgres);

        let SchemaBuilder { foreign_keys, .. } =
            sb.unwrap().build(graphql_schema).unwrap();

        assert_eq!(foreign_keys.len(), 2);
        assert_eq!(foreign_keys[0].create_statement(), "ALTER TABLE namespace_index1.lender ADD CONSTRAINT fk_lender_borrower__borrower_account FOREIGN KEY (borrower) REFERENCES namespace_index1.borrower(account) ON DELETE NO ACTION ON UPDATE NO ACTION INITIALLY DEFERRED;".to_string());
        assert_eq!(foreign_keys[1].create_statement(), "ALTER TABLE namespace_index1.auditor ADD CONSTRAINT fk_auditor_borrower__borrower_account FOREIGN KEY (borrower) REFERENCES namespace_index1.borrower(account) ON DELETE NO ACTION ON UPDATE NO ACTION INITIALLY DEFERRED;".to_string());
    }

    #[test]
    fn test_schema_builder_for_postgres_creates_fk_with_proper_column_names() {
        let graphql_schema: &str = r#"
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

        let sb = SchemaBuilder::new("namespace", "index1", "v1", DbType::Postgres);

        let SchemaBuilder { foreign_keys, .. } =
            sb.unwrap().build(graphql_schema).unwrap();

        assert_eq!(foreign_keys.len(), 2);
        assert_eq!(foreign_keys[0].create_statement(), "ALTER TABLE namespace_index1.message ADD CONSTRAINT fk_message_sender__account_id FOREIGN KEY (sender) REFERENCES namespace_index1.account(id) ON DELETE NO ACTION ON UPDATE NO ACTION INITIALLY DEFERRED;".to_string());
        assert_eq!(foreign_keys[1].create_statement(), "ALTER TABLE namespace_index1.message ADD CONSTRAINT fk_message_receiver__account_id FOREIGN KEY (receiver) REFERENCES namespace_index1.account(id) ON DELETE NO ACTION ON UPDATE NO ACTION INITIALLY DEFERRED;".to_string());
    }

    #[test]
    fn test_get_implicit_foreign_keys_for_schema() {
        let implicit_schema: &str = r#"
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

        let mut expected = HashMap::new();
        expected.insert(
            "lender".to_string(),
            HashMap::from([(
                "borrower".to_string(),
                ("borrower".to_string(), "id".to_string()),
            )]),
        );
        expected.insert(
            "auditor".to_string(),
            HashMap::from([(
                "borrower".to_string(),
                ("borrower".to_string(), "id".to_string()),
            )]),
        );

        let implicit_fks = get_foreign_keys(implicit_schema).unwrap();
        assert_eq!(expected, implicit_fks);
    }

    #[test]
    fn test_get_explicit_foreign_keys_for_schema() {
        let explicit_schema: &str = r#"
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

        let mut expected = HashMap::new();
        expected.insert(
            "lender".to_string(),
            HashMap::from([(
                "borrower".to_string(),
                ("borrower".to_string(), "account".to_string()),
            )]),
        );
        expected.insert(
            "auditor".to_string(),
            HashMap::from([(
                "borrower".to_string(),
                ("borrower".to_string(), "account".to_string()),
            )]),
        );

        let explicit_fks = get_foreign_keys(explicit_schema).unwrap();
        assert_eq!(expected, explicit_fks);
    }
}

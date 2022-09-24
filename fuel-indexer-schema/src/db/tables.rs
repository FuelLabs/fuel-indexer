use crate::db::models::{ColumnIndex, CreateStatement, ForeignKey, IdCol, IndexMethod};
use crate::{get_schema_types, type_id, BASE_SCHEMA};
use fuel_indexer_database::{queries, DbType, IndexerConnection, IndexerConnectionPool};
use fuel_indexer_database_types::*;
use graphql_parser::parse_schema;
use graphql_parser::schema::{Definition, Field, SchemaDefinition, Type, TypeDefinition};
use std::collections::{HashMap, HashSet};

fn normalize_field_type_name(name: &str) -> String {
    let s = name.to_string();
    let mut chars = s.chars();
    chars.next_back();
    chars.as_str().to_string()
}

fn extract_table_name_from_field_type(f: &Field<String>) -> String {
    normalize_field_type_name(&f.field_type.to_string()).to_lowercase()
}

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
                panic!("Error parsing graphql schema {:?}", e)
            }
        };
        let (primitives, _) = get_schema_types(&base_ast);

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
            Err(e) => panic!("Error parsing graphql schema {:?}", e),
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

        let query = query.cloned().expect("TODO: this needs to be error type");

        for def in ast.definitions.iter() {
            if let Definition::TypeDefinition(typ) = def {
                self.generate_table_sql(&query, typ);
            }
        }

        self.query = query;
        self.schema = schema.to_string();

        self
    }

    pub async fn commit_metadata(self, conn: &mut IndexerConnection) -> sqlx::Result<Schema> {
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

        let field_defs = query_fields.get(&query).expect("No query root!");

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

    fn process_type<'a>(&self, field_type: &Type<'a, String>) -> (ColumnType, bool) {
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

    fn process_field_index_directive(
        &self,
        field: &Field<String>,
        column: NewColumn,
        table_name: &String,
    ) -> Option<ColumnIndex> {
        if !field.directives.is_empty() {
            return Some(ColumnIndex {
                db_type: self.db_type.clone(),
                table_name: table_name.to_string(),
                namespace: self.namespace.clone(),
                method: IndexMethod::Btree,
                unique: false,
                column,
            });
        }

        None
    }

    fn generate_columns<'a>(
        &mut self,
        type_id: i64,
        fields: &[Field<'a, String>],
        table_name: &String,
    ) -> String {
        let mut fragments = vec![];

        for (pos, f) in fields.iter().enumerate() {
            let (typ, nullable) = self.process_type(&f.field_type);

            if typ == ColumnType::ForeignKey {
                let fk = ForeignKey::new(
                    self.db_type.clone(),
                    self.namespace.clone(),
                    table_name.clone(),
                    f.name.clone(),
                    extract_table_name_from_field_type(f),
                    IdCol::to_string(),
                );

                let column = NewColumn {
                    type_id,
                    column_position: pos as i32,
                    column_name: f.name.to_string(),
                    column_type: ColumnType::UInt8.to_string(),
                    graphql_type: f.field_type.to_string(),
                    nullable,
                };

                fragments.push(column.sql_fragment());
                self.columns.push(column);
                self.foreign_keys.push(fk);

                continue;
            }

            let column = NewColumn {
                type_id,
                column_position: pos as i32,
                column_name: f.name.to_string(),
                column_type: typ.to_string(),
                graphql_type: f.field_type.to_string(),
                nullable,
            };

            if let Some(ColumnIndex {
                db_type,
                table_name,
                namespace,
                method,
                unique,
                column,
            }) = self.process_field_index_directive(f, column.clone(), table_name)
            {
                self.indices.push(ColumnIndex {
                    db_type,
                    table_name,
                    namespace,
                    method,
                    unique,
                    column,
                });
            }

            fragments.push(column.sql_fragment());
            self.columns.push(column);
        }

        let object_column = NewColumn {
            type_id,
            column_position: fragments.len() as i32,
            column_name: "object".to_string(),
            column_type: "Blob".to_string(),
            graphql_type: "__".into(),
            nullable: false,
        };

        fragments.push(object_column.sql_fragment());
        self.columns.push(object_column);

        fragments.join(",\n")
    }

    fn generate_table_sql<'a>(&mut self, root: &str, typ: &TypeDefinition<'a, String>) {
        fn map_fields(fields: &[Field<'_, String>]) -> HashMap<String, String> {
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
                let columns = self.generate_columns(type_id as i64, &o.fields, &table_name);

                let sql_table = self.db_type.table_name(&self.namespace, &table_name);

                let create = format!(
                    "CREATE TABLE IF NOT EXISTS\n {} (\n {}\n)",
                    sql_table, columns,
                );

                self.statements.push(create);
                self.type_ids.push(TypeId {
                    id: type_id as i64,
                    schema_version: self.version.to_string(),
                    schema_name: self.namespace.to_string(),
                    graphql_name: o.name.to_string(),
                    table_name,
                });
            }
            o => panic!("Got a non-object type! {:?}", o),
        }
    }
}

#[derive(Debug)]
pub struct Schema {
    pub version: String,
    /// Graph ID, and the DB schema this data lives in.
    pub namespace: String,
    /// Root Graphql type.
    pub query: String,
    /// List of GraphQL type names.
    pub types: HashSet<String>,
    /// Mapping of key/value pairs per GraphQL type.
    pub fields: HashMap<String, HashMap<String, String>>,
}

impl Schema {
    pub async fn load_from_db(pool: &IndexerConnectionPool, name: &str) -> sqlx::Result<Self> {
        let mut conn = pool.acquire().await?;
        let root = queries::graph_root_latest(&mut conn, name).await?;
        let root_cols = queries::root_columns_list_by_id(&mut conn, root.id).await?;
        let typeids =
            queries::type_id_list_by_name(&mut conn, &root.schema_name, &root.version).await?;

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

use crate::{db::IndexerSchemaDbResult, QUERY_ROOT};
use async_graphql_parser::types::TypeKind;
use fuel_indexer_database::{
    queries, types::*, DbType, IndexerConnection, IndexerConnectionPool,
};
use fuel_indexer_lib::graphql::ParsedGraphQLSchema;
use fuel_indexer_lib::manifest::Manifest;
use fuel_indexer_lib::ExecutionSource;
use fuel_indexer_types::{graphql::GraphQLSchema, type_id};
use itertools::Itertools;
use std::collections::BTreeMap;

/// IndexerSchema is used to encapsulate most of the logic related to parsing
/// GraphQL types, generating SQL from those types, and committing that SQL to
/// the database.
pub struct IndexerSchema {
    /// The database type.
    db_type: DbType,

    /// The parsed GraphQL schema.
    parsed: ParsedGraphQLSchema,

    /// The GraphQL schema.
    schema: GraphQLSchema,

    /// The tables generated from the GraphQL schema.
    tables: Vec<Table>,

    /// The namespace of the indexer.
    namespace: String,

    /// The identifier of the indexer.
    identifier: String,

    /// The execution source of the indexer.
    exec_source: ExecutionSource,
}

impl IndexerSchema {
    /// Create a new `IndexerSchema`.
    pub fn new(
        namespace: &str,
        identifier: &str,
        schema: &GraphQLSchema,
        db_type: DbType,
        exec_source: ExecutionSource,
    ) -> IndexerSchemaDbResult<Self> {
        Ok(IndexerSchema {
            db_type,
            namespace: namespace.to_string(),
            identifier: identifier.to_string(),
            schema: schema.to_owned(),
            parsed: ParsedGraphQLSchema::default(),
            exec_source,
            tables: Vec::new(),
        })
    }

    /// Return the associated `ParsedGraphQLSchema`.
    pub fn parsed(&self) -> &ParsedGraphQLSchema {
        &self.parsed
    }

    /// Generate table SQL for each indexable object in the given GraphQL schema.
    pub async fn build_and_commit(
        mut self,
        schema: &GraphQLSchema,
        conn: &mut IndexerConnection,
    ) -> IndexerSchemaDbResult<Self> {
        let parsed_schema = ParsedGraphQLSchema::new(
            &self.namespace,
            &self.identifier,
            ExecutionSource::Wasm,
            Some(schema),
        )?;

        let mut statements = Vec::new();
        self.schema = schema.to_owned();
        self.parsed = parsed_schema;

        let root = GraphRoot {
            version: schema.version().to_owned(),
            schema_name: self.namespace.to_owned(),
            schema_identifier: self.identifier.to_owned(),
            schema: self.schema.to_string(),
            ..GraphRoot::default()
        };

        queries::new_graph_root(conn, root).await?;

        let root =
            queries::graph_root_latest(conn, &self.namespace, &self.identifier).await?;

        match self.db_type {
            DbType::Postgres => {
                let create = format!(
                    "CREATE SCHEMA IF NOT EXISTS {}_{};",
                    self.namespace, self.identifier
                );
                statements.push(create);
            }
        }

        let type_ids = self
            .parsed
            .fields_for_typeids()
            .iter()
            .map(|(f, typ_name)| {
                TypeId::from_field_def(
                    typ_name,
                    f,
                    &self.namespace,
                    &self.identifier,
                    self.parsed.schema().version(),
                )
            })
            .unique_by(|t| t.id)
            .collect::<Vec<TypeId>>();

        queries::type_id_insert(conn, type_ids.clone()).await?;

        let type_ids = queries::type_id_list_by_name(
            conn,
            &root.schema_name,
            &root.version,
            &root.schema_identifier,
        )
        .await?;

        let tables = self
            .parsed
            .indexable_objects()
            .iter()
            .filter_map(|typ| match &typ.kind {
                TypeKind::Object(o) => {
                    let col_type_ids = o
                        .fields
                        .iter()
                        .filter_map(|f| {
                            Some(
                                type_ids
                                    .iter()
                                    .find(|t| t.graphql_name == f.node.name.to_string())
                                    .expect(&format!(
                                        "No associated TypeId for field '{}'.",
                                        f.node.name.to_string()
                                    ))
                                    .id,
                            )
                        })
                        .collect::<Vec<i64>>();
                    Some(Table::from_typdef(
                        typ.to_owned(),
                        &self.parsed,
                        col_type_ids,
                    ))
                }
                _ => None,
            })
            .collect::<Vec<Table>>();

        let table_stmnts = tables.iter().map(|t| t.create()).collect::<Vec<String>>();
        statements.extend(table_stmnts);

        let constraint_stmnts = tables
            .iter()
            .flat_map(|t| t.constraints())
            .map(|c| c.create())
            .collect::<Vec<String>>();

        statements.extend(constraint_stmnts);

        for stmnt in statements.iter() {
            queries::execute_query(conn, stmnt.to_owned()).await?;
        }

        self.tables = tables;

        Ok(self)
    }

    /// Load a `Schema` from the database.
    pub async fn load_from_db(
        pool: &IndexerConnectionPool,
        namespace: &str,
        identifier: &str,
    ) -> IndexerSchemaDbResult<Self> {
        // TODO: Might be expensive to always load this from the DB each time. Maybe
        // we can cache and stash this somewhere?

        let mut conn = pool.acquire().await?;
        let root = queries::graph_root_latest(&mut conn, namespace, identifier).await?;
        let type_ids = queries::type_id_list_by_name(
            &mut conn,
            &root.schema_name,
            &root.version,
            identifier,
        )
        .await?;

        let indexer_id =
            queries::get_indexer_id(&mut conn, namespace, identifier).await?;
        let IndexerAssetBundle { manifest, .. } =
            queries::latest_assets_for_indexer(&mut conn, &indexer_id).await?;
        let manifest = Manifest::try_from(&manifest.bytes).expect("Bad manifest.");

        let schema = GraphQLSchema::new(root.schema.clone());
        let parsed = ParsedGraphQLSchema::new(
            namespace,
            identifier,
            manifest.execution_source(),
            Some(&schema),
        )?;

        let tables = parsed
            .indexable_objects()
            .iter()
            .filter_map(|typ| match &typ.kind {
                TypeKind::Object(o) => {
                    let col_type_ids = o
                        .fields
                        .iter()
                        .map(|f| {
                            type_ids
                                .iter()
                                .find(|t| t.graphql_name == f.node.name.to_string())
                                .expect("No associated TypeId for object.")
                                .id
                        })
                        .collect::<Vec<i64>>();
                    Some(Table::from_typdef(typ.to_owned(), &parsed, col_type_ids))
                }
                _ => None,
            })
            .collect::<Vec<Table>>();

        let mut schema = IndexerSchema {
            namespace: root.schema_name,
            identifier: root.schema_identifier,
            schema,

            tables,
            parsed,
            db_type: DbType::Postgres,
            exec_source: manifest.execution_source(),
        };

        schema.register_queryroot_fields();

        Ok(schema)
    }

    // **** HACK ****

    // Below we manually add a `QueryRoot` type, with its corresponding field types
    // data being each `Object` defined in the schema.

    // We need this because at the moment our GraphQL query parsing is tightly-coupled
    // to our old way of resolving GraphQL types (which was using a `QueryType` object
    // defined in a `TypeSystemDefinition::Schema`)

    /// Register the `QueryRoot` type and its corresponding field types.
    pub fn register_queryroot_fields(&mut self) {
        self.parsed.object_field_mappings.insert(
            QUERY_ROOT.to_string(),
            self.parsed
                .objects()
                .keys()
                .map(|k| (k.to_lowercase(), k.clone()))
                .collect::<BTreeMap<String, String>>(),
        );
    }
}

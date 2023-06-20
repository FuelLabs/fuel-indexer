use crate::{db::IndexerSchemaDbResult, parser::ParsedGraphQLSchema, QUERY_ROOT};
use fuel_indexer_database::{
    queries, types::*, DbType, IndexerConnection, IndexerConnectionPool,
};
use fuel_indexer_lib::manifest::Manifest;
use fuel_indexer_lib::ExecutionSource;
use fuel_indexer_types::{graphql::GraphQLSchema, type_id};
use std::collections::BTreeMap;

/// IndexerSchema is used to encapsulate most of the logic related to parsing
/// GraphQL types, generating SQL from those types, and committing that SQL to
/// the database.
#[derive(Default)]
pub struct IndexerSchema {
    db_type: DbType,
    parsed: ParsedGraphQLSchema,
    schema: GraphQLSchema,
    tables: Vec<Table>,
    namespace: String,
    identifier: String,
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
        let parsed =
            ParsedGraphQLSchema::new(namespace, identifier, exec_source.clone(), None)?;

        Ok(IndexerSchema {
            db_type,
            namespace: namespace.to_string(),
            identifier: identifier.to_string(),
            schema: schema.to_owned(),
            parsed,
            exec_source,
            ..Default::default()
        })
    }

    /// Return the associated `ParsedGraphQLSchema`.
    pub fn parsed(&self) -> &ParsedGraphQLSchema {
        &self.parsed
    }

    /// Generate table SQL for each indexable object in the given GraphQL schema.
    pub fn build(mut self, schema: &GraphQLSchema) -> IndexerSchemaDbResult<Self> {
        let parsed_schema = ParsedGraphQLSchema::new(
            &self.namespace,
            &self.identifier,
            ExecutionSource::Wasm,
            Some(schema),
        )?;

        let mut statements = Vec::new();
        self.schema = schema.to_owned();
        self.parsed = parsed_schema;

        match self.db_type {
            DbType::Postgres => {
                let create = format!(
                    "CREATE SCHEMA IF NOT EXISTS {}_{}",
                    self.namespace, self.identifier
                );
                statements.push(create);
            }
        }

        let tables = self
            .parsed
            .indexable_objects()
            .iter()
            .map(|o| Table::from(o.to_owned()))
            .collect::<Vec<Table>>();

        let _table_stmnts = tables.iter().map(|t| t.create()).collect::<Vec<String>>();

        self.tables = tables;

        Ok(self)
    }

    /// Commit all SQL metadata to the database.
    pub async fn commit_sql_metadata(
        self,
        conn: &mut IndexerConnection,
    ) -> IndexerSchemaDbResult<IndexerSchema> {
        let IndexerSchema {
            namespace,
            identifier,
            schema,
            db_type,
            exec_source,
            tables,
            parsed,
        } = self;

        let version = schema.version();

        let root = GraphRoot {
            version: schema.version().to_owned(),
            schema_name: namespace.to_owned(),
            schema_identifier: identifier.to_owned(),
            schema: schema.to_string(),
            ..GraphRoot::default()
        };

        queries::new_graph_root(conn, root).await?;

        let latest = queries::graph_root_latest(conn, &namespace, &identifier).await?;

        let field_defs = parsed.fields_for_columns();

        let cols = field_defs
            .iter()
            .map(|f| RootColumns {
                root_id: latest.id,
                column_name: f.name.to_string(),
                graphql_type: f.name.node.to_string(),
                ..RootColumns::default()
            })
            .collect::<Vec<RootColumns>>();

        let columns = parsed
            .fields_for_columns()
            .iter()
            .map(|f| {
                let type_id = type_id(&namespace, &f.name.node);
                FooColumn::from_field_def(f, &namespace, &identifier, version, type_id)
            })
            .collect::<Vec<FooColumn>>();

        let type_ids = parsed
            .fields_for_columns()
            .iter()
            .map(|f| FooTypeId::from_field_def(f, &namespace, &identifier, version))
            .collect::<Vec<FooTypeId>>();

        queries::new_root_columns(conn, cols).await?;

        queries::foo_type_id_insert(conn, type_ids).await?;
        queries::foo_new_column_insert(conn, columns).await?;

        let mut schema = IndexerSchema {
            db_type: db_type.clone(),
            exec_source: exec_source.clone(),
            parsed: parsed.clone(),
            tables,
            schema: schema.to_owned(),
            namespace: namespace.to_owned(),
            identifier: identifier.to_owned(),
        };

        schema.register_queryroot_fields();

        Ok(schema)
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
        let _type_ids = queries::type_id_list_by_name(
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
            .map(|o| Table::from(o.to_owned()))
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

use crate::{db::IndexerSchemaDbResult, QUERY_ROOT};
use async_graphql_parser::types::TypeKind;
use fuel_indexer_database::{
    queries, types::*, DbType, IndexerConnection, IndexerConnectionPool,
};
use fuel_indexer_lib::graphql::{GraphQLSchema, ParsedGraphQLSchema};
use fuel_indexer_lib::{manifest::Manifest, ExecutionSource};
use itertools::Itertools;
use std::collections::BTreeMap;

/// `IndexerSchema` is used to encapsulate most of the logic related to parsing
/// GraphQL types, generating SQL from those types, and committing that SQL to
/// the database.
#[derive(Default)]
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
}

impl IndexerSchema {
    /// Create a new `IndexerSchema`.
    pub fn new(
        namespace: &str,
        identifier: &str,
        schema: &GraphQLSchema,
        db_type: DbType,
    ) -> IndexerSchemaDbResult<Self> {
        Ok(IndexerSchema {
            db_type,
            namespace: namespace.to_string(),
            identifier: identifier.to_string(),
            schema: schema.to_owned(),
            parsed: ParsedGraphQLSchema::default(),
            tables: Vec::new(),
        })
    }

    /// Return the associated `ParsedGraphQLSchema`.
    pub fn parsed(&self) -> &ParsedGraphQLSchema {
        &self.parsed
    }

    /// Generate table SQL for each indexable object in the given GraphQL schema.
    ///
    /// Ideally all of these queries should return the objects that they persist to the
    /// DB (e.g., `INSERT .. RETURNING *`).
    ///
    /// TODO: We should also be caching as much of this `IndexerSchema` as possible
    pub async fn commit(
        mut self,
        schema: &GraphQLSchema,
        exec_source: ExecutionSource,
        conn: &mut IndexerConnection,
    ) -> IndexerSchemaDbResult<Self> {
        let parsed_schema = ParsedGraphQLSchema::new(
            &self.namespace,
            &self.identifier,
            exec_source,
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

        // TODO: Abstract this into a SQLSchema (or named something else)?
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
            .type_defs()
            .iter()
            .map(|(_, t)| TypeId::from_typdef(t, &self.parsed))
            .unique_by(|t| t.id)
            .collect::<Vec<TypeId>>();

        queries::type_id_insert(conn, type_ids).await?;

        let tables = self
            .parsed
            .type_defs()
            .iter()
            .map(|(_, t)| Table::from_typdef(t, &self.parsed))
            .collect::<Vec<Table>>();

        let columns = tables
            .iter()
            .flat_map(|t| t.columns())
            .map(|c| c.to_owned())
            .collect::<Vec<Column>>();

        queries::new_column_insert(conn, columns).await?;

        let table_stmnts = tables
            .iter()
            .filter_map(|t| {
                let stmnt = t.create();
                if stmnt.is_empty() {
                    return None;
                }
                Some(stmnt)
            })
            .collect::<Vec<String>>();
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

    /// Load a `IndexerSchema` from the database.
    pub async fn load(
        pool: &IndexerConnectionPool,
        namespace: &str,
        identifier: &str,
    ) -> IndexerSchemaDbResult<Self> {
        let mut conn = pool.acquire().await?;
        let root = queries::graph_root_latest(&mut conn, namespace, identifier).await?;

        let indexer_id =
            queries::get_indexer_id(&mut conn, namespace, identifier).await?;
        let IndexerAsset { bytes, .. } = queries::latest_asset_for_indexer(
            &mut conn,
            &indexer_id,
            IndexerAssetType::Manifest,
        )
        .await?;
        let manifest = Manifest::try_from(&bytes)?;

        let schema = GraphQLSchema::new(root.schema.clone());
        let parsed = ParsedGraphQLSchema::new(
            namespace,
            identifier,
            manifest.execution_source(),
            Some(&schema),
        )?;

        let tables = parsed
            .type_defs()
            .iter()
            .filter_map(|(_, typ)| match &typ.kind {
                TypeKind::Object(_o) => Some(Table::from_typdef(typ, &parsed)),
                TypeKind::Union(_u) => Some(Table::from_typdef(typ, &parsed)),
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
        };

        schema.register_queryroot_fields();

        Ok(schema)
    }

    /// Register the `QueryRoot` type and its corresponding field types.
    pub fn register_queryroot_fields(&mut self) {
        // **** HACK ****

        // Below we manually add a `QueryRoot` type, with its corresponding field types
        // data being each `Object` defined in the schema.

        // We need this because at the moment our GraphQL query parsing is tightly-coupled
        // to our old way of resolving GraphQL types (which was using a `QueryType` object
        // defined in a `TypeSystemDefinition::Schema`)
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

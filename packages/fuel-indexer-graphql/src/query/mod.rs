pub(crate) mod arguments;
pub(crate) mod parse;
pub(crate) mod prepare;

use async_graphql::{dynamic::Schema as DynamicSchema, Request};
use async_graphql_parser::{parse_query, types::DocumentOperations, Positioned};
use async_graphql_value::Name;
use fuel_indexer_database::{queries, IndexerConnectionPool};
use fuel_indexer_schema::db::tables::IndexerSchema;
use serde_json::Value;

use crate::{
    query::{parse::ParsedOperation, prepare::prepare_operation},
    GraphqlError, GraphqlResult,
};

use self::arguments::ParamType;

/// Execute user query and return results.
pub async fn execute(
    request: Request,
    dynamic_schema: DynamicSchema,
    user_query: String,
    pool: IndexerConnectionPool,
    schema: IndexerSchema,
) -> GraphqlResult<Value> {
    // Because the schema types from async-graphql expect each field to be resolved
    // separately, it became untenable to use the .execute() method of the dynamic
    // schema itself to resolve queries. Instead, we set it to only resolve
    // introspection queries and then pass any non-introspection queries to our
    // custom query resolver.
    match request.operation_name.as_deref() {
        Some("IntrospectionQuery") | Some("introspectionquery") => {
            let introspection_results = dynamic_schema.execute(request).await;
            let data = introspection_results.data.into_json()?;

            Ok(data)
        }
        Some(_) | None => {
            let exec_doc = parse_query(user_query.as_str())?;
            let queries = match exec_doc.operations {
                DocumentOperations::Single(op_def) => {
                    let parsed = ParsedOperation::generate(
                        schema.parsed(),
                        &op_def.node,
                        &exec_doc.fragments,
                        None,
                        &request.variables,
                    )?;

                    let query = prepare_operation(
                        &parsed,
                        schema.parsed(),
                        &pool.database_type(),
                    )?;
                    vec![query]
                }
                DocumentOperations::Multiple(op_defs) => {
                    let mut queries = vec![];
                    for (name, op_def) in op_defs.iter() {
                        let parsed = ParsedOperation::generate(
                            schema.parsed(),
                            &op_def.node,
                            &exec_doc.fragments,
                            Some(name.to_string()),
                            &request.variables,
                        )?;
                        let prepared = prepare_operation(
                            &parsed,
                            schema.parsed(),
                            &pool.database_type(),
                        )?;
                        queries.push(prepared);
                    }

                    queries
                }
            };

            let mut conn = match pool.acquire().await {
                Ok(c) => c,
                Err(e) => return Err(GraphqlError::QueryError(e.to_string())),
            };

            if queries.len() == 1 {
                let query = &queries[0];
                match queries::run_query(&mut conn, query.to_string()).await {
                    Ok(r) => Ok(r[0].clone()),
                    Err(e) => Err(GraphqlError::QueryError(e.to_string())),
                }
            } else {
                let mut res_map = serde_json::Map::new();
                for query in queries {
                    if let Some(name) = &query.name {
                        match queries::run_query(&mut conn, query.to_string()).await {
                            Ok(r) => {
                                let _ = res_map.insert(name.to_string(), r[0].clone());
                            }
                            Err(e) => {
                                return Err(GraphqlError::QueryError(e.to_string()))
                            }
                        }
                    }
                }

                Ok(serde_json::Value::Object(res_map))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum QueryKind {
    Object,
    Connection,
    Cte,
}

/// The type of selection that can be present in a user's operation.
#[derive(Debug, Clone)]
pub enum ParsedSelection {
    Scalar {
        name: Name,
        parent_entity: String,
        alias: Option<Positioned<Name>>,
    },
    Object {
        name: Name,
        parent_entity: String,
        alias: Option<Positioned<Name>>,
        fields: Vec<ParsedSelection>,
        is_part_of_list: bool,
        arguments: Vec<ParamType>,
        entity_type: String,
    },
    List {
        name: Name,
        alias: Option<Positioned<Name>>,
        node: Box<ParsedSelection>,
        obj_type: String,
    },
    QueryRoot {
        name: Name,
        alias: Option<Positioned<Name>>,
        fields: Vec<ParsedSelection>,
        arguments: Vec<ParamType>,
        kind: QueryKind,
        root_entity_type: String,
    },
    PageInfo {
        name: Name,
        alias: Option<Positioned<Name>>,
        fields: Vec<ParsedSelection>,
        parent_entity: String,
    },
    Edge {
        name: Name,
        cursor: Box<Option<ParsedSelection>>,
        node: Box<Option<ParsedSelection>>,
        entity: String,
    },
}

impl ParsedSelection {
    /// Return name for a `ParsedSelection`.
    pub fn name(&self) -> String {
        match &self {
            ParsedSelection::Scalar { name, .. } => name.to_string(),
            ParsedSelection::Object { name, .. } => name.to_string(),
            ParsedSelection::List { name, .. } => name.to_string(),
            ParsedSelection::QueryRoot { name, .. } => name.to_string(),
            ParsedSelection::PageInfo { name, .. } => name.to_string(),
            ParsedSelection::Edge { name, .. } => name.to_string(),
        }
    }
}

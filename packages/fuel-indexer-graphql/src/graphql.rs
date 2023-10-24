use super::{
    arguments::{parse_argument_into_param, ParamType, QueryParams},
    queries::{JoinCondition, QueryElement, QueryJoinNode, UserQuery},
};
use async_graphql_parser::{
    parse_query,
    types::{
        DocumentOperations, ExecutableDocument, Field, FragmentDefinition,
        FragmentSpread, OperationDefinition, OperationType, SelectionSet, TypeCondition,
    },
};
use fuel_indexer_database_types::DbType;
use fuel_indexer_schema::db::tables::IndexerSchema;
use std::collections::HashMap;
use thiserror::Error;

pub type GraphqlResult<T> = Result<T, GraphqlError>;

#[derive(Debug, Error)]
pub enum GraphqlError {
    #[error("GraphQl Parser error: {0:?}")]
    ParseError(#[from] async_graphql_parser::Error),
    #[error("Error building dynamic schema: {0:?}")]
    DynamicSchemaBuildError(#[from] async_graphql::dynamic::SchemaError),
    #[error("Could not parse introspection response: {0:?}")]
    IntrospectionQueryError(#[from] serde_json::Error),
    #[error("Unrecognized Type: {0:?}")]
    UnrecognizedType(String),
    #[error("Unrecognized Field in {0:?}: {1:?}")]
    UnrecognizedField(String, String),
    #[error("Unrecognized Argument in {0:?}: {1:?}")]
    UnrecognizedArgument(String, String),
    #[error("Operation not supported: {0:?}")]
    OperationNotSupported(String),
    #[error("Fragment for {0:?} can't be used within {1:?}.")]
    InvalidFragmentSelection(Fragment, String),
    #[error("Unsupported Value Type: {0:?}")]
    UnsupportedValueType(String),
    #[error("Failed to resolve query fragments.")]
    FragmentResolverFailed,
    #[error("Selection not supported.")]
    SelectionNotSupported,
    #[error("Unsupported negation for filter type: {0:?}")]
    UnsupportedNegation(String),
    #[error("Filters should have at least one predicate")]
    NoPredicatesInFilter,
    #[error("Unsupported filter operation type: {0:?}")]
    UnsupportedFilterOperation(String),
    #[error("Unable to parse value into string, bool, or i64: {0:?}")]
    UnableToParseValue(String),
    #[error("No available predicates to associate with logical operator")]
    MissingPartnerForBinaryLogicalOperator,
    #[error("Paginated query must have an order applied to at least one field")]
    UnorderedPaginatedQuery,
    #[error("Query error: {0:?}")]
    QueryError(String),
}

#[derive(Clone, Debug)]
pub enum Selection {
    Field {
        name: String,
        params: Vec<ParamType>,
        sub_selections: Selections,
        alias: Option<String>,
    },
    Fragment(String),
}

#[derive(Clone, Debug)]
pub struct Selections {
    has_fragments: bool,
    selections: Vec<Selection>,
}

impl Selections {
    pub fn new(
        schema: &IndexerSchema,
        field_type: Option<&String>,
        set: &SelectionSet,
    ) -> GraphqlResult<Selections> {
        let mut selections = Vec::with_capacity(set.items.len());
        let mut has_fragments = false;

        for item in &set.items {
            match &item.node {
                async_graphql_parser::types::Selection::Field(field) => {
                    // TODO: directives and sub-selections for nested types...
                    let Field {
                        name,
                        selection_set,
                        arguments,
                        alias,
                        ..
                    } = &field.node;
                    let subfield_type =
                        match schema.parsed().graphql_type(field_type, &name.to_string())
                        {
                            Some(typ) => typ,
                            None => {
                                if let Some(field_type) = field_type {
                                    return Err(GraphqlError::UnrecognizedField(
                                        field_type.into(),
                                        name.to_string(),
                                    ));
                                } else {
                                    return Err(GraphqlError::UnrecognizedType(
                                        name.to_string(),
                                    ));
                                }
                            }
                        };

                    let params = arguments
                        .iter()
                        .map(|(arg, value)| {
                            parse_argument_into_param(
                                Some(subfield_type),
                                &arg.to_string(),
                                value.node.clone(),
                                schema,
                            )
                        })
                        .collect::<Result<Vec<ParamType>, GraphqlError>>()?;

                    let sub_selections = Selections::new(
                        schema,
                        Some(subfield_type),
                        &selection_set.node,
                    )?;
                    selections.push(Selection::Field {
                        name: name.to_string(),
                        params,
                        sub_selections,
                        alias: if alias.is_some() {
                            Some(alias.clone().unwrap().to_string())
                        } else {
                            None
                        },
                    });
                }
                async_graphql_parser::types::Selection::FragmentSpread(frag) => {
                    let FragmentSpread { fragment_name, .. } = &frag.node;
                    has_fragments = true;
                    selections.push(Selection::Fragment(fragment_name.to_string()));
                }
                // TODO: Support inline fragments
                _ => return Err(GraphqlError::SelectionNotSupported),
            }
        }

        Ok(Selections {
            has_fragments,
            selections,
        })
    }

    pub fn resolve_fragments(
        &mut self,
        schema: &IndexerSchema,
        cond: Option<&String>,
        fragments: &HashMap<String, Fragment>,
    ) -> GraphqlResult<usize> {
        let mut has_fragments = false;
        let mut resolved = 0;
        let mut selections = Vec::new();

        for selection in &mut self.selections {
            match selection {
                Selection::Fragment(name) => {
                    if let Some(frag) = fragments.get(name) {
                        if !frag.check_cond(cond) {
                            if let Some(c) = cond {
                                return Err(GraphqlError::InvalidFragmentSelection(
                                    frag.clone(),
                                    c.to_string(),
                                ));
                            } else {
                                return Err(GraphqlError::FragmentResolverFailed);
                            }
                        }
                        resolved += 1;
                        selections.extend(frag.selections.get_selections());
                    } else {
                        has_fragments = true;
                        selections.push(Selection::Fragment(name.to_string()));
                    }
                }
                Selection::Field {
                    name,
                    params,
                    sub_selections,
                    alias,
                } => {
                    let field_type =
                        schema.parsed().graphql_type(cond, name).ok_or_else(|| {
                            if let Some(c) = cond {
                                GraphqlError::UnrecognizedField(
                                    c.to_string(),
                                    name.to_string(),
                                )
                            } else {
                                GraphqlError::UnrecognizedType(name.to_string())
                            }
                        })?;
                    let _ = sub_selections.resolve_fragments(
                        schema,
                        Some(&field_type.clone()),
                        fragments,
                    )?;

                    selections.push(Selection::Field {
                        name: name.to_string(),
                        params: params.to_vec(),
                        sub_selections: sub_selections.clone(),
                        alias: alias.clone(),
                    });
                }
            }
        }

        self.selections = selections;
        self.has_fragments = has_fragments;
        Ok(resolved)
    }

    pub fn get_selections(&self) -> Vec<Selection> {
        self.selections.clone()
    }
}

#[derive(Clone, Debug)]
pub struct Fragment {
    cond: String,
    selections: Selections,
}

impl Fragment {
    pub fn new(
        schema: &IndexerSchema,
        cond: String,
        selection_set: &async_graphql_parser::types::SelectionSet,
    ) -> GraphqlResult<Fragment> {
        let selections = Selections::new(schema, Some(&cond), selection_set)?;

        Ok(Fragment { cond, selections })
    }

    pub fn check_cond(&self, cond: Option<&String>) -> bool {
        if let Some(c) = cond {
            self.cond == *c
        } else {
            false
        }
    }

    pub fn has_fragments(&self) -> bool {
        self.selections.has_fragments
    }

    /// Return the number of fragments resolved
    pub fn resolve_fragments(
        &mut self,
        schema: &IndexerSchema,
        fragments: &HashMap<String, Fragment>,
    ) -> GraphqlResult<usize> {
        self.selections
            .resolve_fragments(schema, Some(&self.cond.clone()), fragments)
    }
}

#[derive(Debug)]
pub struct Operation {
    namespace: String,
    identifier: String,
    selections: Selections,
}

impl Operation {
    pub fn new(
        namespace: String,
        identifier: String,
        selections: Selections,
    ) -> Operation {
        Operation {
            namespace,
            identifier,
            selections,
        }
    }

    pub fn parse(&self, schema: &IndexerSchema) -> Vec<UserQuery> {
        let Operation {
            namespace,
            identifier,
            selections,
            ..
        } = self;

        let mut queries = Vec::new();

        for selection in selections.get_selections() {
            let mut elements: Vec<QueryElement> = Vec::new();
            let mut entities: Vec<String> = Vec::new();

            let mut joins: HashMap<String, QueryJoinNode> = HashMap::new();
            let mut query_params: QueryParams = QueryParams::default();

            let mut nested_entity_stack: Vec<String> = Vec::new();

            // Selections can have their own set of subselections and so on, so a queue
            // is created with the first level of selections. In order to track the containing
            // entity of the selection, an entity list of the same length is created.
            if let Selection::Field {
                name: entity_name,
                params: filters,
                sub_selections: selections,
                alias,
            } = selection
            {
                let mut queue: Vec<Selection> = Vec::new();

                // Selections and entities will be popped from their respective vectors
                // easy access to an element. In order to be compliant with the GraphQL
                // spec (which says that a query should be resovled top-down), the order
                // of the elements is reversed prior to insertion in the queues.
                entities.append(
                    &mut vec![entity_name.clone(); selections.selections.len()]
                        .drain(..)
                        .rev()
                        .collect::<Vec<String>>(),
                );
                queue.append(
                    &mut selections
                        .get_selections()
                        .drain(..)
                        .rev()
                        .collect::<Vec<Selection>>(),
                );

                if !filters.is_empty() {
                    query_params.add_params(
                        filters,
                        format!("{namespace}_{identifier}.{entity_name}"),
                    );
                }

                let mut last_seen_entities_len = entities.len();

                while let Some(current) = queue.pop() {
                    let entity_name = entities.pop().unwrap();

                    // If a selection was processed without adding additional selections
                    // to the queue, then check the entity of the selection against the
                    // current nesting level. If they differ, then the operation has moved
                    // out of a child entity into a parent entity.
                    if let Some(current_nesting_level) = nested_entity_stack.last() {
                        if entities.len() < last_seen_entities_len
                            && current_nesting_level != &entity_name
                        {
                            let _ = nested_entity_stack.pop();
                            elements.push(QueryElement::ObjectClosingBoundary);
                        }
                    }

                    last_seen_entities_len = entities.len();

                    if let Selection::Field {
                        name: field_name,
                        params: filters,
                        sub_selections: subselections,
                        alias,
                    } = current
                    {
                        if subselections.selections.is_empty() {
                            elements.push(QueryElement::Field {
                                key: alias.unwrap_or(field_name.clone()),
                                value: format!(
                                    "{namespace}_{identifier}.{entity_name}.{field_name}"
                                ),
                            });
                            if !filters.is_empty() {
                                query_params.add_params(
                                    filters,
                                    format!("{namespace}_{identifier}.{entity_name}"),
                                );
                            }
                        } else {
                            let mut new_entity = field_name.clone();
                            // If the current entity has a foreign key on the current
                            // selection, join the foreign table on that primary key
                            // and set the field as the innermost entity by pushing to the stack.
                            if let Some(field_to_foreign_key) = schema
                                .parsed()
                                .foreign_key_mappings()
                                .get(&entity_name.to_lowercase())
                            {
                                if let Some((foreign_key_table, foreign_key_col)) =
                                    field_to_foreign_key.get(&field_name.to_lowercase())
                                {
                                    let join_condition = JoinCondition {
                                        referencing_key_table: format!(
                                            "{namespace}_{identifier}.{entity_name}"
                                        ),
                                        referencing_key_col: field_name.clone(),
                                        primary_key_table: format!(
                                            "{namespace}_{identifier}.{foreign_key_table}"
                                        ),
                                        primary_key_col: foreign_key_col.clone(),
                                    };

                                    // Joins are modelled like a directed graph in
                                    // order to ensure that tables can be joined in
                                    // a dependent order, if necessary.
                                    match joins
                                        .get_mut(&join_condition.referencing_key_table)
                                    {
                                        Some(join_node) => {
                                            join_node.dependencies.insert(
                                                join_condition.primary_key_table.clone(),
                                                join_condition.clone(),
                                            );
                                        }
                                        None => {
                                            joins.insert(
                                                join_condition
                                                    .referencing_key_table
                                                    .clone(),
                                                QueryJoinNode {
                                                    dependencies: HashMap::from([(
                                                        join_condition
                                                            .primary_key_table
                                                            .clone(),
                                                        join_condition.clone(),
                                                    )]),
                                                    dependents: HashMap::new(),
                                                },
                                            );
                                        }
                                    };

                                    if *foreign_key_table != field_name {
                                        new_entity = foreign_key_table.to_string();
                                    }

                                    match joins.get_mut(&join_condition.primary_key_table)
                                    {
                                        Some(join_node) => {
                                            join_node.dependents.insert(
                                                join_condition
                                                    .referencing_key_table
                                                    .clone(),
                                                join_condition.clone(),
                                            );
                                        }
                                        None => {
                                            joins.insert(
                                                join_condition.primary_key_table.clone(),
                                                QueryJoinNode {
                                                    dependencies: HashMap::new(),
                                                    dependents: HashMap::from([(
                                                        join_condition
                                                            .referencing_key_table
                                                            .clone(),
                                                        join_condition.clone(),
                                                    )]),
                                                },
                                            );
                                        }
                                    };
                                    if !filters.is_empty() {
                                        query_params.add_params(
                                    filters,
                                    format!("{namespace}_{identifier}.{foreign_key_table}"),
                                        );
                                    }
                                }
                            }

                            // Add the subselections and entities to the ends of
                            // their respective vectors so that they are resolved
                            // immediately after their parent selection.
                            entities.append(&mut vec![
                                new_entity.clone();
                                subselections.selections.len()
                            ]);
                            nested_entity_stack.push(new_entity.clone());

                            elements.push(QueryElement::ObjectOpeningBoundary {
                                key: alias.unwrap_or(field_name.clone()),
                            });

                            queue.append(&mut subselections.get_selections());
                        }
                    }
                }

                // If the query document ends without selections from outer entities,
                // then append the requisite number of object closing boundaries in
                // order to properly format the JSON structure for the database query.
                if !nested_entity_stack.is_empty() {
                    elements.append(&mut vec![
                        QueryElement::ObjectClosingBoundary;
                        nested_entity_stack.len()
                    ]);
                }

                let query = UserQuery {
                    elements,
                    joins,
                    namespace_identifier: format!("{namespace}_{identifier}"),
                    entity_name,
                    query_params,
                    alias,
                };

                queries.push(query)
            }
        }

        queries
    }
}

#[derive(Debug)]
pub struct GraphqlQuery {
    operations: Vec<Operation>,
}

impl GraphqlQuery {
    pub fn parse(&self, schema: &IndexerSchema) -> Vec<UserQuery> {
        let queries: Vec<UserQuery> = self
            .operations
            .iter()
            .flat_map(|o| o.parse(schema))
            .collect::<Vec<UserQuery>>();

        queries
    }

    pub fn as_sql(
        &self,
        schema: &IndexerSchema,
        db_type: DbType,
    ) -> Result<Vec<String>, GraphqlError> {
        let queries = self.parse(schema);

        queries
            .into_iter()
            .map(|mut q| q.to_sql(&db_type))
            .collect::<Result<Vec<String>, GraphqlError>>()
    }
}

pub struct GraphqlQueryBuilder<'a> {
    schema: &'a IndexerSchema,
    document: ExecutableDocument,
}

impl<'a> GraphqlQueryBuilder<'a> {
    pub fn new(
        schema: &'a IndexerSchema,
        query: &'a str,
    ) -> GraphqlResult<GraphqlQueryBuilder<'a>> {
        let document = parse_query::<&str>(query)?;
        Ok(GraphqlQueryBuilder { schema, document })
    }

    pub fn build(self) -> GraphqlResult<GraphqlQuery> {
        let fragments = self.process_fragments()?;
        let operations = self.process_operations(fragments)?;
        Ok(GraphqlQuery { operations })
    }

    fn process_operation(
        &self,
        operation: &OperationDefinition,
        fragments: &HashMap<String, Fragment>,
    ) -> GraphqlResult<Operation> {
        match operation.ty {
            OperationType::Query => {
                // TODO: directives and variable definitions....
                let OperationDefinition { selection_set, .. } = operation;
                let mut selections =
                    Selections::new(self.schema, None, &selection_set.node)?;
                selections.resolve_fragments(self.schema, None, fragments)?;

                Ok(Operation::new(
                    self.schema.parsed().namespace().to_string(),
                    self.schema.parsed().identifier().to_string(),
                    selections,
                ))
            }
            OperationType::Mutation => {
                Err(GraphqlError::OperationNotSupported("Mutation".into()))
            }
            OperationType::Subscription => {
                Err(GraphqlError::OperationNotSupported("Subscription".into()))
            }
        }
    }

    fn process_operations(
        &self,
        fragments: HashMap<String, Fragment>,
    ) -> GraphqlResult<Vec<Operation>> {
        let mut operations = vec![];

        match &self.document.operations {
            DocumentOperations::Single(operation_def) => {
                let op = self.process_operation(&operation_def.node, &fragments)?;
                operations.push(op);
            }
            DocumentOperations::Multiple(operation_map) => {
                for (_name, operation_def) in operation_map.iter() {
                    let op = self.process_operation(&operation_def.node, &fragments)?;
                    operations.push(op);
                }
            }
        }

        Ok(operations)
    }

    fn process_fragments(&self) -> GraphqlResult<HashMap<String, Fragment>> {
        let mut fragments = HashMap::new();
        let mut to_resolve = Vec::new();

        for (name, fragment_def) in self.document.fragments.iter() {
            let FragmentDefinition {
                type_condition,
                selection_set,
                ..
            } = &fragment_def.node;

            let TypeCondition { on: cond } = &type_condition.node;

            if !self.schema.parsed().has_type(&cond.to_string()) {
                return Err(GraphqlError::UnrecognizedType(cond.to_string()));
            }

            let frag = Fragment::new(self.schema, cond.to_string(), &selection_set.node)?;

            if frag.has_fragments() {
                to_resolve.push((name.to_string(), frag));
            } else {
                fragments.insert(name.to_string(), frag);
            }
        }

        loop {
            let mut resolved = 0;
            let mut remaining = Vec::new();

            for (name, mut frag) in to_resolve.into_iter() {
                resolved += frag.resolve_fragments(self.schema, &fragments)?;

                if frag.has_fragments() {
                    remaining.push((name, frag))
                } else {
                    fragments.insert(name, frag);
                }
            }

            if !remaining.is_empty() && resolved == 0 {
                return Err(GraphqlError::FragmentResolverFailed);
            } else if remaining.is_empty() {
                break;
            }

            to_resolve = remaining;
        }

        Ok(fragments)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use fuel_indexer_lib::graphql::GraphQLSchema;

    #[test]
    fn test_operation_parse_into_user_query() {
        let selections_on_block_field = Selections {
            has_fragments: false,
            selections: vec![
                Selection::Field {
                    name: "id".to_string(),
                    params: Vec::new(),
                    sub_selections: Selections {
                        has_fragments: false,
                        selections: Vec::new(),
                    },
                    alias: None,
                },
                Selection::Field {
                    name: "height".to_string(),
                    params: Vec::new(),
                    sub_selections: Selections {
                        has_fragments: false,
                        selections: Vec::new(),
                    },
                    alias: None,
                },
            ],
        };

        let selections_on_tx_field = Selections {
            has_fragments: false,
            selections: vec![
                Selection::Field {
                    name: "block".to_string(),
                    params: Vec::new(),
                    sub_selections: selections_on_block_field,
                    alias: None,
                },
                Selection::Field {
                    name: "id".to_string(),
                    params: Vec::new(),
                    sub_selections: Selections {
                        has_fragments: false,
                        selections: Vec::new(),
                    },
                    alias: None,
                },
                Selection::Field {
                    name: "timestamp".to_string(),
                    params: Vec::new(),
                    sub_selections: Selections {
                        has_fragments: false,
                        selections: Vec::new(),
                    },
                    alias: None,
                },
            ],
        };

        let query_selections = vec![Selection::Field {
            name: "tx".to_string(),
            params: Vec::new(),
            sub_selections: selections_on_tx_field,
            alias: None,
        }];

        let operation = Operation {
            namespace: "fuel_indexer_test".to_string(),
            identifier: "test_index".to_string(),
            selections: Selections {
                has_fragments: false,
                selections: query_selections,
            },
        };

        let schema = r#"
type Block @entity {
    id: ID!
    height: U64!
    timestamp: I64!
}

type Tx @entity {
    id: ID!
    timestamp: I64!
    block: Block
    input_data: Json!
}
"#;

        let schema = IndexerSchema::new(
            "fuel_indexer_test",
            "test_index",
            &GraphQLSchema::new(schema.to_string()),
            DbType::Postgres,
        )
        .unwrap();

        let expected = vec![UserQuery {
            elements: vec![
                QueryElement::ObjectOpeningBoundary {
                    key: "block".to_string(),
                },
                QueryElement::Field {
                    key: "height".to_string(),
                    value: "fuel_indexer_test_test_index.block.height".to_string(),
                },
                QueryElement::Field {
                    key: "id".to_string(),
                    value: "fuel_indexer_test_test_index.block.id".to_string(),
                },
                QueryElement::ObjectClosingBoundary,
                QueryElement::Field {
                    key: "id".to_string(),
                    value: "fuel_indexer_test_test_index.tx.id".to_string(),
                },
                QueryElement::Field {
                    key: "timestamp".to_string(),
                    value: "fuel_indexer_test_test_index.tx.timestamp".to_string(),
                },
            ],
            joins: HashMap::from([
                (
                    "fuel_indexer_test_test_index.tx".to_string(),
                    QueryJoinNode {
                        dependencies: HashMap::from([(
                            "fuel_indexer_test_test_index.block".to_string(),
                            JoinCondition {
                                referencing_key_table: "fuel_indexer_test_test_index.tx"
                                    .to_string(),
                                referencing_key_col: "block".to_string(),
                                primary_key_table: "fuel_indexer_test_test_index.block"
                                    .to_string(),
                                primary_key_col: "id".to_string(),
                            },
                        )]),
                        dependents: HashMap::new(),
                    },
                ),
                (
                    "fuel_indexer_test_test_index.block".to_string(),
                    QueryJoinNode {
                        dependents: HashMap::from([(
                            "fuel_indexer_test_test_index.tx".to_string(),
                            JoinCondition {
                                referencing_key_table: "fuel_indexer_test_test_index.tx"
                                    .to_string(),
                                referencing_key_col: "block".to_string(),
                                primary_key_table: "fuel_indexer_test_test_index.block"
                                    .to_string(),
                                primary_key_col: "id".to_string(),
                            },
                        )]),
                        dependencies: HashMap::new(),
                    },
                ),
            ]),
            namespace_identifier: "fuel_indexer_test_test_index".to_string(),
            entity_name: "tx".to_string(),
            query_params: QueryParams::default(),
            alias: None,
        }];
        assert_eq!(expected, operation.parse(&schema));
    }
}

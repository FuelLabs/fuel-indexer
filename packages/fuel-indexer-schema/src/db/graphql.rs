use crate::db::tables::Schema;
use crate::sql_types::{DbType, QueryElement, QueryFilter, UserQuery};
use graphql_parser::query as gql;
use std::collections::HashMap;
use thiserror::Error;

type GraphqlResult<T> = Result<T, GraphqlError>;

#[derive(Debug, Error)]
pub enum GraphqlError {
    #[error("GraphQl Parser error: {0:?}")]
    ParseError(#[from] gql::ParseError),
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
}

#[derive(Clone, Debug)]
pub enum Selection {
    Field(String, Vec<Filter>, Selections),
    Fragment(String),
}

#[derive(Clone, Debug)]
pub struct Filter {
    name: String,
    value: String,
}

impl Filter {
    pub fn new(name: String, value: String) -> Filter {
        Filter { name, value }
    }

    pub fn as_sql(&self, _jsonify: bool) -> String {
        format!("{} = {}", self.name, self.value)
    }
}

#[derive(Clone, Debug)]
pub struct Selections {
    _field_type: String,
    has_fragments: bool,
    selections: Vec<Selection>,
}

impl Selections {
    pub fn new<'a>(
        schema: &Schema,
        field_type: &str,
        set: &gql::SelectionSet<'a, &'a str>,
    ) -> GraphqlResult<Selections> {
        let mut selections = Vec::with_capacity(set.items.len());
        let mut has_fragments = false;

        for item in &set.items {
            match item {
                gql::Selection::Field(field) => {
                    // TODO: directives and sub-selections for nested types...
                    let gql::Field {
                        name,
                        selection_set,
                        arguments,
                        ..
                    } = field;

                    let subfield_type =
                        schema.field_type(field_type, name).ok_or_else(|| {
                            GraphqlError::UnrecognizedField(
                                field_type.into(),
                                name.to_string(),
                            )
                        })?;

                    let mut filters = vec![];
                    for (arg, value) in arguments {
                        if schema.field_type(subfield_type, arg).is_none() {
                            return Err(GraphqlError::UnrecognizedArgument(
                                subfield_type.into(),
                                arg.to_string(),
                            ));
                        }

                        let val = match value {
                            gql::Value::Int(val) => {
                                format!(
                                    "{}",
                                    val.as_i64().expect("Failed to parse value as i64")
                                )
                            }
                            gql::Value::Float(val) => format!("{val}",),
                            gql::Value::String(val) => val.to_string(),
                            gql::Value::Boolean(val) => format!("{val}",),
                            gql::Value::Null => String::from("NULL"),
                            o => {
                                return Err(GraphqlError::UnsupportedValueType(format!(
                                    "{o:#?}",
                                )))
                            }
                        };

                        filters.push(Filter::new(arg.to_string(), val));
                    }

                    let sub_selections =
                        Selections::new(schema, subfield_type, selection_set)?;
                    selections.push(Selection::Field(
                        name.to_string(),
                        filters,
                        sub_selections,
                    ));
                }
                gql::Selection::FragmentSpread(frag) => {
                    let gql::FragmentSpread { fragment_name, .. } = frag;
                    has_fragments = true;
                    selections.push(Selection::Fragment(fragment_name.to_string()));
                }
                // Inline fragments not handled yet....
                _ => return Err(GraphqlError::SelectionNotSupported),
            }
        }

        Ok(Selections {
            _field_type: field_type.to_string(),
            has_fragments,
            selections,
        })
    }

    pub fn resolve_fragments(
        &mut self,
        schema: &Schema,
        cond: &str,
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
                            return Err(GraphqlError::InvalidFragmentSelection(
                                frag.clone(),
                                cond.to_string(),
                            ));
                        }
                        resolved += 1;
                        selections.extend(frag.selections.get_selections());
                    } else {
                        has_fragments = true;
                        selections.push(Selection::Fragment(name.to_string()));
                    }
                }
                Selection::Field(name, filters, sub_selection) => {
                    let field_type = schema
                        .field_type(cond, name)
                        .expect("Unable to retrieve field type");
                    let _ =
                        sub_selection.resolve_fragments(schema, field_type, fragments)?;

                    selections.push(Selection::Field(
                        name.to_string(),
                        filters.to_vec(),
                        sub_selection.clone(),
                    ));
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
    pub fn new<'a>(
        schema: &Schema,
        cond: String,
        selection_set: &gql::SelectionSet<'a, &'a str>,
    ) -> GraphqlResult<Fragment> {
        let selections = Selections::new(schema, &cond, selection_set)?;

        Ok(Fragment { cond, selections })
    }

    pub fn check_cond(&self, cond: &str) -> bool {
        self.cond == cond
    }

    pub fn has_fragments(&self) -> bool {
        self.selections.has_fragments
    }

    /// Return the number of fragments resolved
    pub fn resolve_fragments(
        &mut self,
        schema: &Schema,
        fragments: &HashMap<String, Fragment>,
    ) -> GraphqlResult<usize> {
        self.selections
            .resolve_fragments(schema, &self.cond, fragments)
    }
}

#[derive(Debug)]
pub struct Operation {
    namespace: String,
    identifier: String,
    _name: String,
    selections: Selections,
}

impl Operation {
    pub fn new(
        namespace: String,
        identifier: String,
        name: String,
        selections: Selections,
    ) -> Operation {
        Operation {
            namespace,
            identifier,
            _name: name,
            selections,
        }
    }

    pub fn parse(&self, schema: &Schema) -> Vec<UserQuery> {
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
            let mut joins: Vec<String> = Vec::new();

            let mut nested_entity_stack: Vec<String> = Vec::new();

            // Selections can have their own set of subselections and so on, so a queue
            // is created with the first level of selections. In order to track the containing
            // entity of the selection, an entity list of the same length is created.
            if let Selection::Field(entity_name, filters, selections) = selection {
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

                    if let Selection::Field(field_name, _f, subselections) = current {
                        if subselections.selections.is_empty() {
                            elements.push(QueryElement::Field {
                                key: field_name.clone(),
                                value: format!(
                                    "{namespace}_{identifier}.{entity_name}.{field_name}"
                                ),
                            });
                        } else {
                            // If the current entity has a foreign key on the current
                            // selection, join the foreign table on that primary key
                            // and set the field as the innermost entity by pushing to the stack.
                            if let Some(field_to_foreign_key) =
                                schema.foreign_keys.get(&entity_name.to_lowercase())
                            {
                                if let Some(foreign_key) =
                                    field_to_foreign_key.get(&field_name.to_lowercase())
                                {
                                    let reference_table = fully_qualified_reference_table(
                                        namespace,
                                        identifier,
                                        &field_name,
                                    );
                                    let referencing_key =
                                        fully_qualified_referencing_field(
                                            namespace,
                                            identifier,
                                            &entity_name,
                                            &field_name,
                                        );
                                    let primary_key = fully_qualified_primary_key(
                                        namespace,
                                        identifier,
                                        &field_name,
                                        foreign_key,
                                    );

                                    joins.push(
                                        inner_join_reference_table_on_foreign_key(
                                            &reference_table,
                                            &referencing_key,
                                            &primary_key,
                                        ),
                                    );

                                    nested_entity_stack.push(field_name.clone());
                                }
                            }

                            // Add the subselections and entities to the ends of
                            // their respective vectors so that they are resolved
                            // immediately after their parent selection.
                            entities.append(&mut vec![
                                field_name.clone();
                                subselections.selections.len()
                            ]);

                            queue.append(&mut subselections.get_selections());

                            elements.push(QueryElement::ObjectOpeningBoundary {
                                key: field_name,
                            });
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

                // TODO: Support filtering operations, e.g. <, >, set membership, etc.
                let filters: Vec<QueryFilter> = filters
                    .into_iter()
                    .map(|f| QueryFilter {
                        key: f.name,
                        relation: "=".to_string(),
                        value: f.value,
                    })
                    .collect();

                let query = UserQuery {
                    elements,
                    joins,
                    namespace_identifier: format!("{namespace}_{identifier}"),
                    entity_name,
                    filters,
                };

                queries.push(query)
            }
        }

        queries
    }
}

fn fully_qualified_reference_table(
    namespace: &str,
    identifier: &str,
    field_name: &str,
) -> String {
    format!("{namespace}_{identifier}.{field_name}")
}

fn fully_qualified_referencing_field(
    namespace: &str,
    identifier: &str,
    entity_name: &str,
    field_name: &str,
) -> String {
    format!("{namespace}_{identifier}.{entity_name}.{field_name}")
}

fn fully_qualified_primary_key(
    namespace: &str,
    identifier: &str,
    field_name: &str,
    foreign_key: &str,
) -> String {
    format!("{namespace}_{identifier}.{field_name}.{foreign_key}")
}

fn inner_join_reference_table_on_foreign_key(
    reference_table: &str,
    referencing_key: &str,
    primary_key: &str,
) -> String {
    format!("INNER JOIN {reference_table} ON {referencing_key} = {primary_key}")
}

#[derive(Debug)]
pub struct GraphqlQuery {
    operations: Vec<Operation>,
}

impl GraphqlQuery {
    pub fn parse(&self, schema: &Schema) -> Vec<UserQuery> {
        let queries: Vec<UserQuery> = self
            .operations
            .iter()
            .flat_map(|o| o.parse(schema))
            .collect::<Vec<UserQuery>>();

        queries
    }

    pub fn as_sql(&self, schema: &Schema, db_type: DbType) -> Vec<String> {
        let queries = self.parse(schema);

        queries
            .into_iter()
            .map(|q| q.to_sql(&db_type))
            .collect::<Vec<String>>()
    }
}

pub struct GraphqlQueryBuilder<'a> {
    schema: &'a Schema,
    document: gql::Document<'a, &'a str>,
}

impl<'a> GraphqlQueryBuilder<'a> {
    pub fn new(
        schema: &'a Schema,
        query: &'a str,
    ) -> GraphqlResult<GraphqlQueryBuilder<'a>> {
        let document = gql::parse_query::<&str>(query)?;
        Ok(GraphqlQueryBuilder { schema, document })
    }

    pub fn build(self) -> GraphqlResult<GraphqlQuery> {
        let fragments = self.process_fragments()?;
        let operations = self.process_operations(fragments)?;

        Ok(GraphqlQuery { operations })
    }

    fn process_operation(
        &self,
        operation: &gql::OperationDefinition<'a, &'a str>,
        fragments: &HashMap<String, Fragment>,
    ) -> GraphqlResult<Operation> {
        match operation {
            gql::OperationDefinition::SelectionSet(set) => {
                let selections = Selections::new(self.schema, &self.schema.query, set)?;

                Ok(Operation::new(
                    self.schema.namespace.clone(),
                    self.schema.identifier.clone(),
                    "Unnamed".into(),
                    selections,
                ))
            }
            gql::OperationDefinition::Query(q) => {
                // TODO: directives and variable definitions....
                let gql::Query {
                    name,
                    selection_set,
                    ..
                } = q;
                let name = name.map_or_else(|| "Unnamed".into(), |o| o.into());

                let mut selections =
                    Selections::new(self.schema, &self.schema.query, selection_set)?;
                selections.resolve_fragments(
                    self.schema,
                    &self.schema.query,
                    fragments,
                )?;

                Ok(Operation::new(
                    self.schema.namespace.clone(),
                    self.schema.identifier.clone(),
                    name,
                    selections,
                ))
            }
            gql::OperationDefinition::Mutation(_) => {
                Err(GraphqlError::OperationNotSupported("Mutation".into()))
            }
            gql::OperationDefinition::Subscription(_) => {
                Err(GraphqlError::OperationNotSupported("Subscription".into()))
            }
        }
    }

    fn process_operations(
        &self,
        fragments: HashMap<String, Fragment>,
    ) -> GraphqlResult<Vec<Operation>> {
        let mut operations = vec![];

        for def in &self.document.definitions {
            if let gql::Definition::Operation(operation) = def {
                let op = self.process_operation(operation, &fragments)?;

                operations.push(op);
            }
        }

        Ok(operations)
    }

    fn process_fragments(&self) -> GraphqlResult<HashMap<String, Fragment>> {
        let mut fragments = HashMap::new();
        let mut to_resolve = Vec::new();

        for def in &self.document.definitions {
            if let gql::Definition::Fragment(frag) = def {
                let gql::FragmentDefinition {
                    name,
                    type_condition,
                    selection_set,
                    ..
                } = frag;

                let gql::TypeCondition::On(cond) = type_condition;

                if !self.schema.check_type(cond) {
                    return Err(GraphqlError::UnrecognizedType(cond.to_string()));
                }

                let frag = Fragment::new(self.schema, cond.to_string(), selection_set)?;

                if frag.has_fragments() {
                    to_resolve.push((name.to_string(), frag));
                } else {
                    fragments.insert(name.to_string(), frag);
                }
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
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_operation_parse_into_user_query() {
        let selections_on_block_field = Selections {
            _field_type: "Block".to_string(),
            has_fragments: false,
            selections: vec![
                Selection::Field(
                    "id".to_string(),
                    Vec::new(),
                    Selections {
                        _field_type: "ID!".to_string(),
                        has_fragments: false,
                        selections: Vec::new(),
                    },
                ),
                Selection::Field(
                    "height".to_string(),
                    Vec::new(),
                    Selections {
                        _field_type: "UInt8!".to_string(),
                        has_fragments: false,
                        selections: Vec::new(),
                    },
                ),
            ],
        };

        let selections_on_tx_field = Selections {
            _field_type: "Tx".to_string(),
            has_fragments: false,
            selections: vec![
                Selection::Field(
                    "block".to_string(),
                    Vec::new(),
                    selections_on_block_field,
                ),
                Selection::Field(
                    "id".to_string(),
                    Vec::new(),
                    Selections {
                        _field_type: "ID!".to_string(),
                        has_fragments: false,
                        selections: Vec::new(),
                    },
                ),
                Selection::Field(
                    "timestamp".to_string(),
                    Vec::new(),
                    Selections {
                        _field_type: "Int8!".to_string(),
                        has_fragments: false,
                        selections: Vec::new(),
                    },
                ),
            ],
        };

        let query_selections = vec![Selection::Field(
            "tx".to_string(),
            Vec::new(),
            selections_on_tx_field,
        )];

        let operation = Operation {
            _name: "".to_string(),
            namespace: "fuel_indexer_test".to_string(),
            identifier: "test_index".to_string(),
            selections: Selections {
                _field_type: "QueryRoot".to_string(),
                has_fragments: false,
                selections: query_selections,
            },
        };

        let fields = HashMap::from([
            (
                "QueryRoot".to_string(),
                HashMap::from([
                    ("tx".to_string(), "Tx".to_string()),
                    ("block".to_string(), "Block".to_string()),
                ]),
            ),
            (
                "Tx".to_string(),
                HashMap::from([
                    ("timestamp".to_string(), "Int8!".to_string()),
                    ("input_data".to_string(), "Json!".to_string()),
                    ("id".to_string(), "ID!".to_string()),
                    ("object".to_string(), "__".to_string()),
                    ("block".to_string(), "Block".to_string()),
                ]),
            ),
            (
                "Block".to_string(),
                HashMap::from([
                    ("id".to_string(), "ID!".to_string()),
                    ("height".to_string(), "UInt8!".to_string()),
                    ("object".to_string(), "__".to_string()),
                    ("timestamp".to_string(), "Int8!".to_string()),
                ]),
            ),
        ]);

        let foreign_keys = HashMap::from([(
            "tx".to_string(),
            HashMap::from([("block".to_string(), "id".to_string())]),
        )]);

        let schema = Schema {
            version: "test_version".to_string(),
            namespace: "fuel_indexer_test".to_string(),
            identifier: "test_index".to_string(),
            query: "QueryRoot".to_string(),
            types: HashSet::from([
                "Tx".to_string(),
                "Block".to_string(),
                "QueryRoot".to_string(),
            ]),
            fields,
            foreign_keys,
        };

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
            joins: vec!["INNER JOIN fuel_indexer_test_test_index.block ON fuel_indexer_test_test_index.tx.block = fuel_indexer_test_test_index.block.id".to_string()],
            namespace_identifier: "fuel_indexer_test_test_index".to_string(),
            entity_name: "tx".to_string(),
            filters: Vec::new()
        }];
        assert_eq!(expected, operation.parse(&schema));
    }
}

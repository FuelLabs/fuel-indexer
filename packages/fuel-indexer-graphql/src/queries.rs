use super::{arguments::QueryParams, graphql::GraphqlError};
use fuel_indexer_database::DbType;

use std::{collections::HashMap, fmt::Display};

/// Represents a part of a user query. Each part can be a key-value pair
/// describing an entity field and its corresponding database table, or a
/// boundary for a nested object; opening boundaries contain a string to
/// be used as a JSON key in the final database query.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum QueryElement {
    Field { key: String, value: String },
    ObjectOpeningBoundary { key: String },
    ObjectClosingBoundary,
}

/// Represents the tables and columns used in a particular database join.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct JoinCondition {
    pub referencing_key_table: String,
    pub referencing_key_col: String,
    pub primary_key_table: String,
    pub primary_key_col: String,
}

impl Display for JoinCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{} = {}.{}",
            self.referencing_key_table,
            self.referencing_key_col,
            self.primary_key_table,
            self.primary_key_col
        )
    }
}

/// Represents a node in a directed acyclic graph (DAG) and used to
/// allow for the sorting of table joins.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct QueryJoinNode {
    pub dependencies: HashMap<String, JoinCondition>,
    pub dependents: HashMap<String, JoinCondition>,
}

/// Represents the full amount of requested information from a user query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserQuery {
    /// The individal parts or tokens of what will become a selection statement.
    pub elements: Vec<QueryElement>,

    /// Contains information about the dependents and dependencies of a particular table join.
    pub joins: HashMap<String, QueryJoinNode>,

    /// The full isolated namespace in which an indexer's entity tables reside.
    pub namespace_identifier: String,

    /// The top-level entity contained in a query.
    pub entity_name: String,

    /// The full set of parameters that can be applied to a query.
    pub query_params: QueryParams,

    // An optional user-suppled alias for an entity field.
    pub alias: Option<String>,
}

impl UserQuery {
    /// Returns the query as a database-specific SQL query.
    pub fn to_sql(&mut self, db_type: &DbType) -> Result<String, GraphqlError> {
        // Different database solutions have unique ways of
        // constructing JSON-formatted queries and results.
        match db_type {
            DbType::Postgres => {
                let selections = self.parse_query_elements_into_selections(db_type);

                let selections_str = selections.join("");

                let sorted_joins = self.get_topologically_sorted_joins();

                let mut last_seen_primary_key_table = "".to_string();
                let mut joins: Vec<String> = Vec::new();

                // For each clause in the list of topologically-sorted joins,
                // check if the clause's primary key table matches the last primary key
                // key table that was seen in this loop. If so, add the join condition to
                // the last join condition; if not, push this clause into the list of joins.
                // This is required because Postgres does not allow for joined primary key tables
                // to be mentioned multiple times.
                for sj in sorted_joins {
                    if sj.primary_key_table == last_seen_primary_key_table {
                        if let Some(elem) = joins.last_mut() {
                            *elem = format!("{elem} AND {sj}")
                        }
                    } else {
                        joins.push(format!(
                            "INNER JOIN {} ON {}",
                            sj.primary_key_table, sj
                        ));
                        last_seen_primary_key_table = sj.primary_key_table;
                    }
                }

                let joins_str = if !joins.is_empty() {
                    joins.join(" ")
                } else {
                    "".to_string()
                };

                // If there's a limit applied to the query, then we need to create a query
                // with pagination info. Otherwise, we can return the entire result set.
                let query: String = if let Some(limit) = self.query_params.limit {
                    // Paginated queries must have an order applied to at least one field.
                    if !self.query_params.sorts.is_empty() {
                        self.create_query_with_paginated(
                            db_type,
                            selections_str,
                            joins_str,
                            limit,
                        )
                    } else {
                        return Err(GraphqlError::UnorderedPaginatedQuery);
                    }
                } else {
                    format!(
                        "SELECT json_build_object({}) FROM {}.{} {} {} {}",
                        selections_str,
                        self.namespace_identifier,
                        self.entity_name,
                        joins_str,
                        self.query_params.get_filtering_expression(db_type),
                        self.query_params.get_ordering_modififer(db_type)
                    )
                };

                Ok(query)
            }
        }
    }

    /// Returns a SQL query that contains the requested results and a Paginated object.
    fn create_query_with_paginated(
        &self,
        db_type: &DbType,
        selections_str: String,
        joins_str: String,
        limit: u64,
    ) -> String {
        // In order to create information about pagination, we need to calculate
        // values according to the amount of records, current offset, and requested
        // limit. To avoid sending additional queries for every request sent to
        // the API, we leverage a common table expression (CTE) which is a table
        // that exists only for the duration of the query and allows us to refer
        // to its result set.
        match db_type {
            db_type @ DbType::Postgres => {
                let json_selections_str =
                    self.get_json_selections_from_cte(db_type).join(",");

                let selection_cte = format!(
                    r#"WITH selection_cte AS (
                        SELECT json_build_object({}) AS {}
                        FROM {}.{}
                        {}
                        {}
                        {}),"#,
                    selections_str,
                    self.entity_name,
                    self.namespace_identifier,
                    self.entity_name,
                    joins_str,
                    self.query_params.get_filtering_expression(db_type),
                    self.query_params.get_ordering_modififer(db_type),
                );

                let total_count_cte =
                    "total_count_cte AS (SELECT COUNT(*) as count FROM selection_cte)"
                        .to_string();

                let offset = self.query_params.offset.unwrap_or(0);
                let alias = self.alias.clone().unwrap_or(self.entity_name.clone());

                let selection_query = format!(
                    r#"SELECT json_build_object(
                        'page_info', json_build_object(
                            'has_next_page', (({limit} + {offset}) < (SELECT count from total_count_cte)),
                            'limit', {limit},
                            'offset', {offset},
                            'pages', ceil((SELECT count from total_count_cte)::float / {limit}::float),
                            'total_count', (SELECT count from total_count_cte)
                        ),
                        '{alias}', (
                            SELECT json_agg(item)
                            FROM (
                                SELECT {json_selections_str} FROM selection_cte
                                LIMIT {limit} OFFSET {offset}
                            ) item
                        )
                    );"#
                );

                [selection_cte, total_count_cte, selection_query].join("\n")
            }
        }
    }

    /// Parses QueryElements into a list of strings that can be used to create a selection statement.
    ///
    /// Each database type should have a way to return result sets as a JSON-friendly structure,
    /// as JSON is the most used format for GraphQL responses.
    fn parse_query_elements_into_selections(&self, db_type: &DbType) -> Vec<String> {
        let mut peekable_elements = self.elements.iter().peekable();

        let mut selections = Vec::new();

        match db_type {
            DbType::Postgres => {
                while let Some(e) = peekable_elements.next() {
                    match e {
                        // Set the key for this JSON element to the name of the entity field
                        // and the value to the corresponding database table so that it can
                        // be successfully retrieved.
                        QueryElement::Field { key, value } => {
                            selections.push(format!("'{key}', {value}"));

                            // If the next element is not a closing boundary, then a comma should
                            // be added so that the resultant SQL query can be properly constructed.
                            if let Some(next_element) = peekable_elements.peek() {
                                match next_element {
                                    QueryElement::Field { .. }
                                    | QueryElement::ObjectOpeningBoundary { .. } => {
                                        selections.push(", ".to_string());
                                    }
                                    _ => {}
                                }
                            }
                        }

                        // If the element is an object opener boundary, then we need to set a
                        // key so that the recipient can properly refer to the nested object.
                        QueryElement::ObjectOpeningBoundary { key } => {
                            selections.push(format!("'{key}', json_build_object("))
                        }

                        QueryElement::ObjectClosingBoundary => {
                            selections.push(")".to_string());

                            if let Some(next_element) = peekable_elements.peek() {
                                match next_element {
                                    QueryElement::Field { .. }
                                    | QueryElement::ObjectOpeningBoundary { .. } => {
                                        selections.push(", ".to_string());
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }

        selections
    }

    /// Returns a list of strings that can be used to select user-requested
    /// elements from a query leveraging common table expressions.
    fn get_json_selections_from_cte(&self, db_type: &DbType) -> Vec<String> {
        let mut selections = Vec::new();

        match db_type {
            DbType::Postgres => {
                let mut peekable_elements = self.elements.iter().peekable();
                let mut nesting_level = 0;

                while let Some(element) = peekable_elements.next() {
                    match element {
                        QueryElement::Field { key, .. } => {
                            selections.push(format!(
                                "{}->'{}' AS {}",
                                self.entity_name, key, key
                            ));
                        }

                        QueryElement::ObjectOpeningBoundary { key } => {
                            selections.push(format!(
                                "{}->'{}' AS {}",
                                self.entity_name, key, key
                            ));
                            nesting_level += 1;

                            // Since we've added the entire sub-object (and its potential
                            // sub-objects) to our selections, we can safely ignore all
                            // fields and objects until we've come back to the top level.
                            for inner_element in peekable_elements.by_ref() {
                                match inner_element {
                                    QueryElement::ObjectOpeningBoundary { .. } => {
                                        nesting_level += 1;
                                    }
                                    QueryElement::ObjectClosingBoundary => {
                                        nesting_level -= 1;
                                        if nesting_level == 0 {
                                            break;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }

                        QueryElement::ObjectClosingBoundary => {}
                    }
                }
            }
        }

        selections
    }

    /// Returns table joins sorted in topological order.
    ///
    /// Some databases (i.e Postgres) require that dependent tables be joined after the tables
    /// the tables they depend upon, i.e. the tables needs to be topologically sorted.
    fn get_topologically_sorted_joins(&mut self) -> Vec<JoinCondition> {
        let mut start_nodes: Vec<String> = self
            .joins
            .iter()
            .filter(|(_k, v)| v.dependencies.is_empty())
            .map(|(k, _v)| k.clone())
            .collect();

        let mut sorted_joins: Vec<JoinCondition> = Vec::new();

        // For each node that does not depend on another node, iterate through their dependents
        // and remove current_node from their dependencies. If all the dependencies of a node
        // have been removed, add it to start_nodes and start the process again.
        while let Some(current_node) = start_nodes.pop() {
            if let Some(node) = self.joins.get_mut(&current_node) {
                for (dependent_node, _) in node.clone().dependents.iter() {
                    if let Some(or) = self.joins.get_mut(dependent_node) {
                        if let Some(dependency) = or.dependencies.remove(&current_node) {
                            sorted_joins.push(dependency);
                            if or.dependencies.is_empty() {
                                start_nodes.push(dependent_node.clone());
                            }
                        }
                    }
                }
            }
        }

        sorted_joins.into_iter().rev().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::arguments::{Filter, FilterType, ParsedValue};

    #[test]
    fn test_user_query_parse_query_elements() {
        let elements = vec![
            QueryElement::Field {
                key: "flat_field_key".to_string(),
                value: "flat_value".to_string(),
            },
            QueryElement::ObjectOpeningBoundary {
                key: "nested_object_key".to_string(),
            },
            QueryElement::Field {
                key: "nested_field_key".to_string(),
                value: "nested_field_value".to_string(),
            },
            QueryElement::ObjectClosingBoundary,
        ];
        let uq = UserQuery {
            elements,
            joins: HashMap::new(),
            namespace_identifier: "".to_string(),
            entity_name: "".to_string(),
            query_params: QueryParams::default(),
            alias: None,
        };

        let expected = vec![
            "'flat_field_key', flat_value".to_string(),
            ", ".to_string(),
            "'nested_object_key', json_build_object(".to_string(),
            "'nested_field_key', nested_field_value".to_string(),
            ")".to_string(),
        ];

        assert_eq!(
            expected,
            uq.parse_query_elements_into_selections(&DbType::Postgres)
        );
    }

    #[test]
    fn test_user_query_to_sql() {
        let elements = vec![
            QueryElement::Field {
                key: "hash".to_string(),
                value: "name_ident.block.hash".to_string(),
            },
            QueryElement::ObjectOpeningBoundary {
                key: "tx".to_string(),
            },
            QueryElement::Field {
                key: "hash".to_string(),
                value: "name_ident.tx.hash".to_string(),
            },
            QueryElement::ObjectClosingBoundary,
            QueryElement::Field {
                key: "height".to_string(),
                value: "name_ident.block.height".to_string(),
            },
        ];

        let mut uq = UserQuery {
            elements,
            joins: HashMap::from([
                (
                    "name_ident.block".to_string(),
                    QueryJoinNode {
                        dependencies: HashMap::new(),
                        dependents: HashMap::from([(
                            "name_ident.tx".to_string(),
                            JoinCondition {
                                referencing_key_table: "name_ident.tx".to_string(),
                                referencing_key_col: "block".to_string(),
                                primary_key_table: "name_ident.block".to_string(),
                                primary_key_col: "id".to_string(),
                            },
                        )]),
                    },
                ),
                (
                    "name_ident.tx".to_string(),
                    QueryJoinNode {
                        dependents: HashMap::new(),
                        dependencies: HashMap::from([(
                            "name_ident.block".to_string(),
                            JoinCondition {
                                referencing_key_table: "name_ident.tx".to_string(),
                                referencing_key_col: "block".to_string(),
                                primary_key_table: "name_ident.block".to_string(),
                                primary_key_col: "id".to_string(),
                            },
                        )]),
                    },
                ),
            ]),
            namespace_identifier: "name_ident".to_string(),
            entity_name: "entity_name".to_string(),
            query_params: QueryParams {
                filters: vec![Filter {
                    fully_qualified_table_name: "name_ident.entity_name".to_string(),
                    filter_type: FilterType::IdSelection(ParsedValue::Number(1)),
                }],
                sorts: vec![],
                offset: None,
                limit: None,
            },
            alias: None,
        };

        let expected = "SELECT json_build_object('hash', name_ident.block.hash, 'tx', json_build_object('hash', name_ident.tx.hash), 'height', name_ident.block.height) FROM name_ident.entity_name INNER JOIN name_ident.block ON name_ident.tx.block = name_ident.block.id WHERE  name_ident.entity_name.id = 1 "
            .to_string();
        assert_eq!(expected, uq.to_sql(&DbType::Postgres).unwrap());
    }
}

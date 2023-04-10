use super::arguments::Filter;
use fuel_indexer_database::DbType;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum QueryElement {
    Field { key: String, value: String },
    ObjectOpeningBoundary { key: String },
    ObjectClosingBoundary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityFilter {
    pub fully_qualified_table_name: String,
    pub filters: Vec<Filter>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct JoinCondition {
    pub referencing_key_table: String,
    pub referencing_key_col: String,
    pub primary_key_table: String,
    pub primary_key_col: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct QueryJoinNode {
    pub dependencies: HashMap<String, JoinCondition>,
    pub dependents: HashMap<String, JoinCondition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserQuery {
    pub elements: Vec<QueryElement>,
    pub joins: HashMap<String, QueryJoinNode>,
    pub namespace_identifier: String,
    pub entity_name: String,
    pub filters: Vec<EntityFilter>,
}

impl UserQuery {
    pub fn to_sql(&mut self, db_type: &DbType) -> String {
        // Different database solutions have unique ways of
        // constructing JSON-formatted queries and results.
        match db_type {
            DbType::Postgres => {
                let elements = self.parse_query_elements(db_type);

                let elements_string = elements.join("");

                let sorted_joins = self.get_topologically_sorted_joins();

                let mut last_seen_primary_key_table = "".to_string();
                let mut joins: Vec<String> = Vec::new();

                for sj in sorted_joins {
                    if sj.primary_key_table == last_seen_primary_key_table {
                        if let Some(elem) = joins.last_mut() {
                            let join_condition = format!(
                                "{}.{} = {}.{}",
                                sj.referencing_key_table,
                                sj.referencing_key_col,
                                sj.primary_key_table,
                                sj.primary_key_col
                            );
                            *elem = format!("{elem} AND {join_condition}")
                        }
                    } else {
                        joins.push(format!(
                            "INNER JOIN {} ON {}.{} = {}.{}",
                            sj.primary_key_table,
                            sj.referencing_key_table,
                            sj.referencing_key_col,
                            sj.primary_key_table,
                            sj.primary_key_col
                        ));
                        last_seen_primary_key_table = sj.primary_key_table;
                    }
                }

                let mut query = format!(
                    "SELECT json_build_object({}) FROM {}.{}",
                    elements_string, self.namespace_identifier, self.entity_name,
                );

                if !joins.is_empty() {
                    query = format!("{query} {}", joins.join(" "))
                }

                if !self.filters.is_empty() {
                    let filter_str = self
                        .filters
                        .iter()
                        .flat_map(|qf| {
                            qf.filters.iter().map(|f| {
                                f.to_sql(qf.fully_qualified_table_name.clone(), db_type)
                            })
                        })
                        .collect::<Vec<String>>()
                        .join(" AND ");
                    query = format!("{query} WHERE {filter_str}");
                }

                println!("{query}");
                query
            }
        }
    }

    fn parse_query_elements(&self, db_type: &DbType) -> Vec<String> {
        let mut peekable_elements = self.elements.iter().peekable();

        let mut elements = Vec::new();

        match db_type {
            DbType::Postgres => {
                while let Some(e) = peekable_elements.next() {
                    match e {
                        // Set the key for this JSON element to the name of the entity field
                        // and the value to the corresponding database table so that it can
                        // be successfully retrieved.
                        QueryElement::Field { key, value } => {
                            elements.push(format!("'{key}', {value}"));

                            // If the next element is not a closing boundary, then a comma should
                            // be added so that the resultant SQL query can be properly constructed.
                            if let Some(next_element) = peekable_elements.peek() {
                                match next_element {
                                    QueryElement::Field { .. }
                                    | QueryElement::ObjectOpeningBoundary { .. } => {
                                        elements.push(", ".to_string());
                                    }
                                    _ => {}
                                }
                            }
                        }

                        // Set a nested JSON object as the value for this entity field.
                        QueryElement::ObjectOpeningBoundary { key } => {
                            elements.push(format!("'{key}', json_build_object("))
                        }

                        QueryElement::ObjectClosingBoundary => {
                            elements.push(")".to_string());

                            if let Some(next_element) = peekable_elements.peek() {
                                match next_element {
                                    QueryElement::Field { .. }
                                    | QueryElement::ObjectOpeningBoundary { .. } => {
                                        elements.push(", ".to_string());
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }

        elements
    }

    fn get_topologically_sorted_joins(&mut self) -> Vec<JoinCondition> {
        let mut yet_to_process =
            self.joins.clone().into_keys().collect::<HashSet<String>>();
        let mut start_nodes: Vec<String> = self
            .joins
            .iter()
            .filter(|(_k, v)| v.dependencies.is_empty())
            .map(|(k, _v)| k.clone())
            .collect();

        let mut sorted_joins: Vec<JoinCondition> = Vec::new();

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

            yet_to_process.remove(&current_node);
        }

        sorted_joins.into_iter().rev().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::db::arguments::ParsedValue;

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
            filters: Vec::new(),
        };

        let expected = vec![
            "'flat_field_key', flat_value".to_string(),
            ", ".to_string(),
            "'nested_object_key', json_build_object(".to_string(),
            "'nested_field_key', nested_field_value".to_string(),
            ")".to_string(),
        ];

        assert_eq!(expected, uq.parse_query_elements(&DbType::Postgres));
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
            filters: vec![EntityFilter {
                fully_qualified_table_name: "name_ident.entity_name".to_string(),
                filters: vec![Filter::IdSelection(ParsedValue::Number(1))],
            }],
        };

        let expected = "SELECT json_build_object('hash', name_ident.block.hash, 'tx', json_build_object('hash', name_ident.tx.hash), 'height', name_ident.block.height) FROM name_ident.entity_name INNER JOIN name_ident.block ON name_ident.tx.block = name_ident.block.id WHERE name_ident.entity_name.id = 1"
            .to_string();
        assert_eq!(expected, uq.to_sql(&DbType::Postgres));
    }
}

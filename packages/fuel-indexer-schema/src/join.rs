use crate::FtColumn;
use fuel_indexer_lib::join_table_typedefs_name;
use serde::{Deserialize, Serialize};

extern crate alloc;

/// Details for the many-to-many relationship.
///
/// This is essentially the same as `fuel_indexer_lib::graphql::parser::JoinTableRelation`, just
/// a bit more compile-time friendly.
#[derive(Debug, Clone)]
pub struct JoinMetadata<'a> {
    /// Name of join table.
    pub table_name: &'a str,

    /// Fully qualified GraphQL namespace for indexer.
    pub namespace: &'a str,

    /// Name of parent table's column on which join is applied.
    pub parent_column_name: &'a str,

    /// Name of child table's column on which join is applied.
    pub child_column_name: &'a str,

    /// Position in the parent's set of `FtColumn`s, at which the many-to-many child column is found.
    pub child_position: usize,
}

/// A raw SQL query.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RawQuery(pub String);

impl From<RawQuery> for Vec<u8> {
    fn from(query: RawQuery) -> Self {
        query.0.into_bytes()
    }
}

impl RawQuery {
    pub fn query(&self) -> &str {
        &self.0
    }

    /// Whether or not there are actual records to insert for this query.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Create a new `RawQuery` from the given metadata and columns.
    pub fn from_metadata(metadata: &JoinMetadata<'_>, columns: &[FtColumn]) -> Self {
        let JoinMetadata {
            table_name,
            namespace,
            parent_column_name,
            child_column_name,
            child_position,
        } = metadata;

        let (parent_typedef_name, child_typedef_name) =
            join_table_typedefs_name(table_name);
        let mut query = format!(
            "INSERT INTO {namespace}.{table_name} ({parent_typedef_name}_{parent_column_name}, {child_typedef_name}_{child_column_name}) VALUES "
        );

        let id_index: usize = columns
            .iter()
            .position(|x| matches!(x, FtColumn::ID(_)))
            .expect("ID field is required for many-to-many relationships.");

        let id = match columns[id_index] {
            FtColumn::ID(Some(id)) => id,
            _ => panic!("No ID field found on Entity."),
        };

        let list_type_field = &columns[*child_position];
        match list_type_field {
            FtColumn::Array(list) => {
                if let Some(list) = list {
                    list.iter().for_each(|item| {
                        query.push_str(
                            format!(" ({}, {}),", id, item.query_fragment()).as_str(),
                        );
                    });
                }
            }
            _ => panic!("Expected array type for many-to-many relationship."),
        }

        // If we didn't actually push any records...
        if query.ends_with("VALUES ") {
            query = "".to_string();
        }

        if !query.is_empty() {
            // Trim the trailing comma
            query.pop();
            query.push_str(&format!(
                    " ON CONFLICT({parent_typedef_name}_{parent_column_name}, {child_typedef_name}_{child_column_name}) DO NOTHING;"
                ));
        }

        Self(query)
    }
}

impl std::fmt::Display for RawQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

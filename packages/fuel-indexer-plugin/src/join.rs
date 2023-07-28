use fuel_indexer_lib::join_table_typedefs_name;
use fuel_indexer_schema::FtColumn;

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

/// The query to insert many-to-many relationships.
pub struct ManyToManyQuery {
    /// The literal SQL query.
    query: String,
}

impl ManyToManyQuery {
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Whether or not there are actual records to insert for this query.
    pub fn is_empty(&self) -> bool {
        self.query.is_empty()
    }

    /// Create a new `ManyToManyQuery` from the given metadata and columns.
    pub fn from_metadata(
        metadata: Vec<JoinMetadata<'_>>,
        columns: Vec<FtColumn>,
    ) -> Self {
        let first = &metadata[0];

        let JoinMetadata {
            table_name,
            namespace,
            parent_column_name,
            child_column_name,
            ..
        } = first;

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

        metadata.iter().for_each(|m| {
            let JoinMetadata { child_position, .. } = m;
            let list_type_field = &columns[*child_position];
            match list_type_field {
                FtColumn::Array(list) => {
                    if let Some(list) = list {
                        if list.is_empty() {
                            query = "".to_string();
                            return;
                        }

                        list.iter().for_each(|item| {
                            query.push_str(
                                format!(" ({}, {}),", id, item.query_fragment()).as_str(),
                            );
                        });
                    } else {
                        query = "".to_string();
                    }
                }
                _ => panic!("Expected array type for many-to-many relationship."),
            }
        });

        if !query.is_empty() {
            // Trim the trailing comma
            query.pop();
            query.push_str(&format!(
                    " ON CONFLICT({parent_typedef_name}_{parent_column_name}, {child_typedef_name}_{child_column_name}) DO NOTHING;"
                ));
        }

        ManyToManyQuery { query }
    }
}

impl From<ManyToManyQuery> for Vec<u8> {
    fn from(query: ManyToManyQuery) -> Self {
        query.query.into_bytes()
    }
}

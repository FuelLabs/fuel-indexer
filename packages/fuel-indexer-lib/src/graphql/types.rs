/// Represents an ID field in a SQL column or GraphQL schema.s
pub struct IdCol;
impl IdCol {
    pub fn to_lowercase_string() -> String {
        "id".to_string()
    }

    pub fn to_lowercase_str() -> &'static str {
        "id"
    }

    pub fn to_uppercase_string() -> String {
        "ID".to_string()
    }

    pub fn to_uppercase_str() -> &'static str {
        "ID"
    }
}

/// Represents a `TypeDefinition`s `object` field.
pub struct ObjectCol;
impl ObjectCol {
    pub fn to_lowercase_string() -> String {
        "object".to_string()
    }

    pub fn to_lowercase_str() -> &'static str {
        "object"
    }

    pub fn to_string() -> String {
        "Object".to_string()
    }

    pub fn to_str() -> &'static str {
        "Object"
    }
}

use crate::api::{ApiError, ApiResult};
use sqlparser::{dialect::GenericDialect, parser::Parser};

/// A validator for SQL queries.
///
/// Intended to ensure that users posting raw SQL queries to web API endpoints
/// are not attempting to do anything malicious.
pub struct SqlQueryValidator {
    /// The SQL dialect to use when parsing queries.
    dialect: GenericDialect,
}

impl SqlQueryValidator {
    /// Creates a new validator.
    pub fn new() -> Self {
        Self {
            dialect: GenericDialect {},
        }
    }

    fn verify_is_select_only(&self, query: &str) -> ApiResult<()> {
        let ast = Parser::parse_sql(&self.dialect, query)?;
        Ok(())
    }
}

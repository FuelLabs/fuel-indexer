use crate::api::{ApiError, ApiResult};
use sqlparser::{dialect::GenericDialect, parser::Parser};

/// A validator for SQL queries.
///
/// Intended to ensure that users posting raw SQL queries to web API endpoints
/// are not attempting to do anything malicious.
pub struct SqlQueryValidator;

impl SqlQueryValidator {
    pub fn verify_is_select_only(&self, query: &str) -> ApiResult<()> {
        let dialect = GenericDialect {};
        let ast = Parser::parse_sql(&dialect, query)?;
        println!(">> AST: {:?}", ast);
        Ok(())
    }
}

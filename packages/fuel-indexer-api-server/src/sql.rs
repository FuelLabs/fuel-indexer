use sqlparser::{ast::Statement, dialect::GenericDialect, parser::Parser};
use thiserror::Error;

/// Result type returned by `SqlQueryValidator`.
type SqlValidatorResult<T> = Result<T, SqlValidatorError>;

/// Error type returned by `SqlQueryValidator`.
#[derive(Debug, Error)]
pub enum SqlValidatorError {
    #[error("Operation is not supported.")]
    OperationNotSupported,
    #[error("SqlParser error: {0:?}")]
    SqlParser(#[from] sqlparser::parser::ParserError),
}

/// A validator for SQL queries.
///
/// Intended to ensure that users posting raw SQL queries to web API endpoints
/// are not attempting to do anything malicious.
pub struct SqlQueryValidator;

impl SqlQueryValidator {
    /// Validates a SQL query.
    pub fn validate_sql_query(query: &str) -> SqlValidatorResult<()> {
        let dialect = GenericDialect {};
        let ast = Parser::parse_sql(&dialect, query)?;
        for stmtnt in ast.iter() {
            match stmtnt {
                Statement::Analyze { .. }
                | Statement::Truncate { .. }
                | Statement::Msck { .. }
                | Statement::Insert { .. }
                | Statement::Directory { .. }
                | Statement::Copy { .. }
                | Statement::CopyIntoSnowflake { .. }
                | Statement::Close { .. }
                | Statement::Update { .. }
                | Statement::Delete { .. }
                | Statement::CreateView { .. }
                | Statement::CreateTable { .. }
                | Statement::CreateVirtualTable { .. }
                | Statement::CreateIndex { .. }
                | Statement::CreateRole { .. }
                | Statement::AlterTable { .. }
                | Statement::AlterIndex { .. }
                | Statement::Drop { .. }
                | Statement::DropFunction { .. }
                | Statement::Declare { .. }
                | Statement::Fetch { .. }
                | Statement::Discard { .. }
                | Statement::SetRole { .. }
                | Statement::SetVariable { .. }
                | Statement::SetTimeZone { .. }
                | Statement::SetNames { .. }
                | Statement::SetNamesDefault { .. }
                | Statement::ShowFunctions { .. }
                | Statement::ShowVariable { .. }
                | Statement::ShowVariables { .. }
                | Statement::ShowCreate { .. }
                | Statement::ShowColumns { .. }
                | Statement::ShowTables { .. }
                | Statement::ShowCollation { .. }
                | Statement::Use { .. }
                | Statement::StartTransaction { .. }
                | Statement::SetTransaction { .. }
                | Statement::Comment { .. }
                | Statement::Commit { .. }
                | Statement::Rollback { .. }
                | Statement::CreateSchema { .. }
                | Statement::CreateDatabase { .. }
                | Statement::CreateFunction { .. }
                | Statement::CreateProcedure { .. }
                | Statement::CreateMacro { .. }
                | Statement::CreateStage { .. }
                | Statement::Assert { .. }
                | Statement::Grant { .. }
                | Statement::Revoke { .. }
                | Statement::Deallocate { .. }
                | Statement::Execute { .. }
                | Statement::Prepare { .. }
                | Statement::Kill { .. }
                | Statement::ExplainTable { .. }
                | Statement::Explain { .. }
                | Statement::Savepoint { .. }
                | Statement::Merge { .. }
                | Statement::Cache { .. }
                | Statement::UNCache { .. }
                | Statement::CreateSequence { .. }
                | Statement::CreateType { .. } => {
                    return Err(SqlValidatorError::OperationNotSupported);
                }
                Statement::Query { .. } => {}
            }
        }

        Ok(())
    }
}

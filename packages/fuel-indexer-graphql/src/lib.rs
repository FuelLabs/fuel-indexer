pub mod dynamic;
pub mod query;

use thiserror::Error;
pub type GraphqlResult<T> = Result<T, GraphqlError>;

#[derive(Debug, Error)]
pub enum GraphqlError {
    #[error("GraphQL parser error: {0:?}")]
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
    #[error("Querying for the entire range is not supported")]
    NoCompleteRangeQueriesAllowed,
    #[error("{0:?}")]
    QueryError(String),
    #[error("Scalar fields require a parent object")]
    ScalarsRequireAParent,
    #[error("Lists of lists are not permitted")]
    ListsOfLists,
    #[error("Could not get base entity type for field: {0:?}")]
    CouldNotGetBaseEntityType(String),
    #[error("Undefined variable: {0:?}")]
    UndefinedVariable(String),
    #[error("Cannot have cycles in query")]
    NoCyclesAllowedInQuery,
    #[error("Root level node should be a query type")]
    RootNeedsToBeAQuery,
    #[error("Improper reference to root name")]
    RootNameOnNonRootObj,
    #[error("Parsing error with internal type - {0:?}")]
    InternalTypeParseError(String),
    #[error("Object query requires an ID argument")]
    ObjectQueryNeedsIdArg,
}

use super::{graphql::GraphqlError, tables::Schema};

use fuel_indexer_database::DbType;
use graphql_parser::query::Value;
use std::{collections::BTreeMap, fmt};

/// Represents the full set of parameters that can be applied to a query.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct QueryParams {
    pub filters: Vec<Filter>,
    pub sorts: Vec<Sort>,
    pub offset: Option<u64>,
    pub limit: Option<u64>,
}

impl QueryParams {
    /// Iterate through the list of parsed parameters
    /// and add them to the corresponding field
    pub(crate) fn add_params(
        &mut self,
        params: Vec<ParamType>,
        fully_qualified_table_name: String,
    ) {
        for param in params {
            match param {
                ParamType::Filter(f) => self.filters.push(Filter {
                    fully_qualified_table_name: fully_qualified_table_name.clone(),
                    filter_type: f,
                }),
                ParamType::Sort(field, order) => self.sorts.push(Sort {
                    fully_qualified_table_name: format!(
                        "{}.{}",
                        fully_qualified_table_name, field
                    ),
                    order,
                }),
                ParamType::Offset(n) => self.offset = Some(n),
                ParamType::Limit(n) => self.limit = Some(n),
            }
        }
    }

    pub(crate) fn to_sql(&self, db_type: &DbType) -> String {
        let mut query_clause = "".to_string();

        if !self.filters.is_empty() {
            let where_expressions = self
                .filters
                .iter()
                .map(|f| f.to_sql(db_type))
                .collect::<Vec<String>>()
                .join(" AND ");
            query_clause =
                ["WHERE".to_string(), query_clause, where_expressions].join(" ");
        }

        if !self.sorts.is_empty() {
            let sort_expressions = self
                .sorts
                .iter()
                .map(|s| format!("{} {}", s.fully_qualified_table_name, s.order))
                .collect::<Vec<String>>()
                .join(", ");
            query_clause =
                [query_clause, "ORDER BY".to_string(), sort_expressions].join(" ");
        }

        if let Some(limit) = self.limit {
            query_clause =
                [query_clause, "LIMIT".to_string(), limit.to_string()].join(" ");
        }

        if let Some(offset) = self.offset {
            query_clause =
                [query_clause, "OFFSET".to_string(), offset.to_string()].join(" ");
        }

        query_clause
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filter {
    pub fully_qualified_table_name: String,
    pub filter_type: FilterType,
}

impl Filter {
    pub fn to_sql(&self, db_type: &DbType) -> String {
        self.filter_type
            .to_sql(self.fully_qualified_table_name.clone(), db_type)
    }
}

/// Represents the different types of parameters that can be created.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParamType {
    Filter(FilterType),
    Sort(String, SortOrder),
    Offset(u64),
    Limit(u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sort {
    pub fully_qualified_table_name: String,
    pub order: SortOrder,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl fmt::Display for SortOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortOrder::Asc => write!(f, "ASC"),
            SortOrder::Desc => write!(f, "DESC"),
        }
    }
}

/// ParsedValue represents the possible value types
/// that the indexer's GraphQL API supports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedValue {
    BigNumber(u128),
    Number(u64),
    String(String),
    Boolean(bool),
}

/// Display trait implementation, mainly to be able to use `.to_string()`.
///
/// Databases may support several value types in a filtering clause, e.g.
/// one can check equality against a number or a string. As such, the
/// `.to_string()` method for each type returns the value in the requisite format.
impl fmt::Display for ParsedValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BigNumber(bn) => {
                write!(f, "{bn}")
            }
            Self::Boolean(b) => {
                write!(f, "{b}")
            }
            Self::Number(n) => {
                write!(f, "{n}")
            }
            Self::String(s) => {
                write!(f, "\'{s}\'")
            }
        }
    }
}

/// Represents an operation through which records can be included or excluded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterType {
    IdSelection(ParsedValue),
    Comparison(Comparison),
    Membership(Membership),
    NullValueCheck(NullValueCheck),
    LogicOp(LogicOp),
}

/// Represents an operation in which a record is compared against a particular value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Comparison {
    Between(String, ParsedValue, ParsedValue),
    Greater(String, ParsedValue),
    GreaterEqual(String, ParsedValue),
    Less(String, ParsedValue),
    LessEqual(String, ParsedValue),
    Equals(String, ParsedValue),
    NotEquals(String, ParsedValue),
}

/// Represents an operation in which a records value is checked for membership in a set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Membership {
    In(String, Vec<ParsedValue>),
    NotIn(String, Vec<ParsedValue>),
}

/// Represents an operation in which records are filtered by the presence of null values or lack thereof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NullValueCheck {
    NoNulls(Vec<String>),
    OnlyNulls(Vec<String>),
}

/// Represents an operation in which filters are to associated and evaluated together.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogicOp {
    And(Box<FilterType>, Box<FilterType>),
    Or(Box<FilterType>, Box<FilterType>),
    Not(Box<FilterType>),
}

impl FilterType {
    /// Returns a string to be used as part of a SQL database query.
    pub fn to_sql(&self, fully_qualified_table: String, db_type: &DbType) -> String {
        match db_type {
            DbType::Postgres => match self {
                Self::Comparison(c) => match c {
                    Comparison::Between(field, min, max) => {
                        format!("{fully_qualified_table}.{field} BETWEEN {min} AND {max}",)
                    }
                    Comparison::Equals(field, val) => {
                        format!("{fully_qualified_table}.{field} = {val}",)
                    }
                    Comparison::NotEquals(field, val) => {
                        format!("{fully_qualified_table}.{field} <> {val}",)
                    }
                    Comparison::Greater(field, val) => {
                        format!("{fully_qualified_table}.{field} > {val}",)
                    }
                    Comparison::GreaterEqual(field, val) => {
                        format!("{fully_qualified_table}.{field} >= {val}",)
                    }
                    Comparison::Less(field, val) => {
                        format!("{fully_qualified_table}.{field} < {val}",)
                    }
                    Comparison::LessEqual(field, val) => {
                        format!("{fully_qualified_table}.{field} <= {val}",)
                    }
                },
                Self::IdSelection(id) => {
                    format!("{fully_qualified_table}.id = {id}")
                }
                Self::LogicOp(lo) => match lo {
                    LogicOp::And(r1, r2) => format!(
                        "({} AND {})",
                        r1.to_sql(fully_qualified_table.clone(), db_type),
                        r2.to_sql(fully_qualified_table, db_type)
                    ),
                    LogicOp::Or(r1, r2) => format!(
                        "({} OR {})",
                        r1.to_sql(fully_qualified_table.clone(), db_type),
                        r2.to_sql(fully_qualified_table, db_type)
                    ),
                    // The NOT logical operator does not get turned into a string as
                    // it will have already been used to transform a filter into its
                    // inverse equivalent.
                    _ => "".to_string(),
                },
                Self::Membership(m) => match m {
                    Membership::In(field, member_set) => {
                        format!(
                            "{fully_qualified_table}.{field} IN ({})",
                            member_set
                                .iter()
                                .map(|v| v.to_string())
                                .collect::<Vec<String>>()
                                .join(", ")
                        )
                    }
                    Membership::NotIn(field, member_set) => {
                        format!(
                            "{fully_qualified_table}.{field} NOT IN ({})",
                            member_set
                                .iter()
                                .map(|v| v.to_string())
                                .collect::<Vec<String>>()
                                .join(", ")
                        )
                    }
                },
                Self::NullValueCheck(nvc) => match nvc {
                    NullValueCheck::NoNulls(column_list) => {
                        return column_list
                            .iter()
                            .map(|col| {
                                format!("{fully_qualified_table}.{col} IS NOT NULL")
                            })
                            .collect::<Vec<String>>()
                            .join(" AND ");
                    }
                    NullValueCheck::OnlyNulls(column_list) => {
                        return column_list
                            .iter()
                            .map(|col| format!("{fully_qualified_table}.{col} IS NULL"))
                            .collect::<Vec<String>>()
                            .join(" AND ");
                    }
                },
            },
        }
    }
}

/// Invert a filter into its opposite filter.
///
/// Each filter should have a inverse type when inverted in order to minimize
/// disruption to the user. When adding a new filter type, special consideration
/// should be given as to if and how it can be represented in the inverse.
impl FilterType {
    fn invert(&self) -> Result<FilterType, GraphqlError> {
        match self {
            FilterType::IdSelection(_) => Err(GraphqlError::UnsupportedNegation(
                "ID selection".to_string(),
            )),
            FilterType::Comparison(c) => match c {
                Comparison::Between(field, val1, val2) => {
                    Ok(FilterType::LogicOp(LogicOp::And(
                        Box::new(FilterType::Comparison(Comparison::Less(
                            field.clone(),
                            val1.clone(),
                        ))),
                        Box::new(FilterType::Comparison(Comparison::Greater(
                            field.clone(),
                            val2.clone(),
                        ))),
                    )))
                }
                Comparison::Greater(field, val) => Ok(FilterType::Comparison(
                    Comparison::LessEqual(field.clone(), val.clone()),
                )),
                Comparison::GreaterEqual(field, val) => Ok(FilterType::Comparison(
                    Comparison::Less(field.clone(), val.clone()),
                )),
                Comparison::Less(field, val) => Ok(FilterType::Comparison(
                    Comparison::GreaterEqual(field.clone(), val.clone()),
                )),
                Comparison::LessEqual(field, val) => Ok(FilterType::Comparison(
                    Comparison::Greater(field.clone(), val.clone()),
                )),
                Comparison::Equals(field, val) => Ok(FilterType::Comparison(
                    Comparison::NotEquals(field.clone(), val.clone()),
                )),
                Comparison::NotEquals(field, val) => Ok(FilterType::Comparison(
                    Comparison::Equals(field.clone(), val.clone()),
                )),
            },
            FilterType::Membership(mf) => match mf {
                Membership::In(field, element_list) => Ok(FilterType::Membership(
                    Membership::NotIn(field.clone(), element_list.clone()),
                )),
                Membership::NotIn(field, element_list) => Ok(FilterType::Membership(
                    Membership::In(field.clone(), element_list.clone()),
                )),
            },
            FilterType::NullValueCheck(nvc) => match nvc {
                NullValueCheck::NoNulls(column_list) => Ok(FilterType::NullValueCheck(
                    NullValueCheck::OnlyNulls(column_list.clone()),
                )),
                NullValueCheck::OnlyNulls(column_list) => Ok(FilterType::NullValueCheck(
                    NullValueCheck::NoNulls(column_list.clone()),
                )),
            },
            FilterType::LogicOp(lo) => match lo {
                LogicOp::And(r1, r2) => Ok(FilterType::LogicOp(LogicOp::And(
                    Box::new(r1.clone().invert()?),
                    Box::new(r2.clone().invert()?),
                ))),
                LogicOp::Or(r1, r2) => Ok(FilterType::LogicOp(LogicOp::Or(
                    Box::new(r1.clone().invert()?),
                    Box::new(r2.clone().invert()?),
                ))),
                LogicOp::Not(f) => Ok(*f.clone()),
            },
        }
    }
}

/// Parse an argument key-value pair into a `Filter`.
///
/// `parse_arguments` is the entry point for parsing all API query arguments.
/// Any new top-level operators should first be added here.
pub fn parse_argument_into_param<'a>(
    entity_type: &String,
    arg: &str,
    value: Value<'a, &'a str>,
    schema: &Schema,
) -> Result<ParamType, GraphqlError> {
    match arg {
        "filter" => {
            // We instantiate an Option<Filter> in order to keep track of the last
            // seen filter in the event that an AND/OR operator is used; if so, the
            // prior filter is associated with the inner filter of the logical operator.
            let mut prior_filter: Option<FilterType> = None;

            if let Value::Object(obj) = value {
                let filter =
                    parse_filter_object(obj, entity_type, schema, &mut prior_filter)?;
                Ok(ParamType::Filter(filter))
            } else {
                Err(GraphqlError::UnsupportedValueType(value.to_string()))
            }
        }
        "id" => Ok(ParamType::Filter(FilterType::IdSelection(parse_value(
            &value,
        )?))),
        "order" => {
            if let Value::Object(obj) = value {
                if let Some((sort_order, predicate)) = obj.into_iter().next() {
                    if let Value::Enum(field) = predicate {
                        if schema.field_type(entity_type, field).is_some() {
                            match sort_order {
                                "asc" => {
                                    return Ok(ParamType::Sort(
                                        field.to_string(),
                                        SortOrder::Asc,
                                    ))
                                }
                                "desc" => {
                                    return Ok(ParamType::Sort(
                                        field.to_string(),
                                        SortOrder::Desc,
                                    ))
                                }
                                other => {
                                    return Err(GraphqlError::UnableToParseValue(
                                        other.to_string(),
                                    ))
                                }
                            }
                        } else {
                            return Err(GraphqlError::UnrecognizedField(
                                entity_type.to_string(),
                                field.to_string(),
                            ));
                        }
                    } else {
                        return Err(GraphqlError::UnsupportedValueType(
                            predicate.to_string(),
                        ));
                    }
                }
                Err(GraphqlError::NoPredicatesInFilter)
            } else {
                Err(GraphqlError::UnsupportedValueType(value.to_string()))
            }
        }
        "offset" => {
            if let Value::Int(offset) = value {
                Ok(ParamType::Offset(offset.as_u64()))
            } else {
                Err(GraphqlError::UnsupportedValueType(value.to_string()))
            }
        }
        "first" => {
            if let Value::Int(limit) = value {
                Ok(ParamType::Limit(limit.as_u64()))
            } else {
                Err(GraphqlError::UnsupportedValueType(value.to_string()))
            }
        }
        _ => Err(GraphqlError::UnrecognizedArgument(
            entity_type.to_string(),
            arg.to_string(),
        )),
    }
}

/// Parse an object from a parsed GraphQL document into a `Filter`.
///
/// This serves as a helper function for starting
/// the parsing operation for values under the "filter" key.
fn parse_filter_object<'a>(
    obj: BTreeMap<&'a str, Value<'a, &'a str>>,
    entity_type: &String,
    schema: &Schema,
    prior_filter: &mut Option<FilterType>,
) -> Result<FilterType, GraphqlError> {
    let mut iter = obj.into_iter();

    if let Some((key, predicate)) = iter.next() {
        return parse_arg_pred_pair(
            key,
            predicate,
            entity_type,
            schema,
            prior_filter,
            &mut iter,
        );
    }
    Err(GraphqlError::NoPredicatesInFilter)
}

/// Parse an argument's key and value (known here as a predicate) into a `Filter`.
///
/// `parse_arg_pred_pair` contains the majority of the filter parsing functionality.
/// Any argument key that matches one of the non-field-specific filtering keywords
/// (i.e. "has" or a logical operator) is decoded accordingly. If the key does not
/// match to the aforementioned keywords but is a field of the entity type, then the
/// key and inner value are parsed into a filter.
fn parse_arg_pred_pair<'a>(
    key: &str,
    predicate: Value<'a, &'a str>,
    entity_type: &String,
    schema: &Schema,
    prior_filter: &mut Option<FilterType>,
    top_level_arg_value_iter: &mut impl Iterator<Item = (&'a str, Value<'a, &'a str>)>,
) -> Result<FilterType, GraphqlError> {
    match key {
        "has" => {
            if let Value::List(elements) = predicate {
                let mut column_list = vec![];
                for element in elements {
                    if let Value::Enum(column) = element {
                        if schema.field_type(entity_type, column).is_some() {
                            column_list.push(column.to_string())
                        } else {
                            return Err(GraphqlError::UnrecognizedField(
                                entity_type.to_string(),
                                column.to_string(),
                            ));
                        }
                    } else {
                        return Err(GraphqlError::UnsupportedValueType(
                            element.to_string(),
                        ));
                    }
                }
                Ok(FilterType::NullValueCheck(NullValueCheck::NoNulls(
                    column_list,
                )))
            } else {
                Err(GraphqlError::UnsupportedValueType(predicate.to_string()))
            }
        }
        "and" | "or" => parse_binary_logical_operator(
            key,
            predicate.clone(),
            entity_type,
            schema,
            top_level_arg_value_iter,
            prior_filter,
        ),

        // "NOT" is a unary logical operator, meaning that it only operates on one component.
        // Thus, we attempt to parse the inner object into a filter and then transform it into
        // its inverse.
        "not" => {
            if let Value::Object(inner_obj) = predicate {
                parse_filter_object(inner_obj, entity_type, schema, prior_filter)?
                    .invert()
            } else {
                Err(GraphqlError::UnsupportedValueType(predicate.to_string()))
            }
        }
        other => {
            if schema.field_type(entity_type, other).is_some() {
                if let Value::Object(inner_obj) = predicate {
                    for (key, predicate) in inner_obj.iter() {
                        match *key {
                            "between" => {
                                if let Value::Object(complex_comparison_obj) = predicate {
                                    if let (Some(min), Some(max)) = (
                                        complex_comparison_obj.get("min"),
                                        complex_comparison_obj.get("max"),
                                    ) {
                                        let (min, max) =
                                            (parse_value(min)?, parse_value(max)?);
                                        return Ok(FilterType::Comparison(
                                            Comparison::Between(
                                                other.to_string(),
                                                min,
                                                max,
                                            ),
                                        ));
                                    }
                                }
                            }
                            "equals" => {
                                return Ok(FilterType::Comparison(Comparison::Equals(
                                    other.to_string(),
                                    parse_value(predicate)?,
                                )))
                            }
                            "gt" => {
                                return Ok(FilterType::Comparison(Comparison::Greater(
                                    other.to_string(),
                                    parse_value(predicate)?,
                                )))
                            }
                            "gte" => {
                                return Ok(FilterType::Comparison(
                                    Comparison::GreaterEqual(
                                        other.to_string(),
                                        parse_value(predicate)?,
                                    ),
                                ));
                            }
                            "lt" => {
                                return Ok(FilterType::Comparison(Comparison::Less(
                                    other.to_string(),
                                    parse_value(predicate)?,
                                )))
                            }
                            "lte" => {
                                return Ok(FilterType::Comparison(Comparison::LessEqual(
                                    other.to_string(),
                                    parse_value(predicate)?,
                                )))
                            }
                            "in" => {
                                if let Value::List(elements) = predicate {
                                    let parsed_elements = elements
                                            .iter()
                                            .map(parse_value)
                                            .collect::<Result<Vec<ParsedValue>,GraphqlError>>();
                                    if let Ok(elements) = parsed_elements {
                                        return Ok(FilterType::Membership(
                                            Membership::In(other.to_string(), elements),
                                        ));
                                    } else {
                                        return Err(GraphqlError::UnableToParseValue(
                                            predicate.to_string(),
                                        ));
                                    }
                                }
                            }
                            _ => {
                                return Err(GraphqlError::UnsupportedFilterOperation(
                                    key.to_string(),
                                ))
                            }
                        }
                    }
                    Err(GraphqlError::NoPredicatesInFilter)
                } else {
                    Err(GraphqlError::UnsupportedValueType(predicate.to_string()))
                }
            } else {
                Err(GraphqlError::UnrecognizedField(
                    entity_type.to_string(),
                    other.to_string(),
                ))
            }
        }
    }
}

/// Parse logical operators that operate on two components.
///
/// `parse_binary_logical_operator` is a special parsing operation that
/// essentially folds or flattens two filters into a single filter. This
/// is also where the nested filtering functionality as filters can be
/// nested arbitrarily deep due to the LogicOp filter type containing filters itself.
fn parse_binary_logical_operator<'a>(
    key: &str,
    predicate: Value<'a, &'a str>,
    entity_type: &String,
    schema: &Schema,
    top_level_arg_value_iter: &mut impl Iterator<Item = (&'a str, Value<'a, &'a str>)>,
    prior_filter: &mut Option<FilterType>,
) -> Result<FilterType, GraphqlError> {
    if let Value::Object(inner_obj) = predicate {
        // Construct the filter contained in the object value for the binary logical operator
        let filter = parse_filter_object(inner_obj, entity_type, schema, prior_filter)?;

        // If we've already constructed a filter prior to this, associate it with
        // the filter that was just parsed from the inner object.
        if let Some(prior_filter) = prior_filter {
            match key {
                "and" => Ok(FilterType::LogicOp(LogicOp::And(
                    Box::new(prior_filter.clone()),
                    Box::new(filter),
                ))),
                "or" => Ok(FilterType::LogicOp(LogicOp::Or(
                    Box::new(prior_filter.clone()),
                    Box::new(filter),
                ))),
                // parse_binary_logical_operator is only called when the key is "and" or "or"
                _ => unreachable!(),
            }
        // It's possible that we may parse a logical operator before we've constructed
        // another filter; this is due to the underlying argument value type being a
        // BTreeMap, which sorts keys alphabetically. If so, get the next top-level
        // key-object-value pair, parse it into a filter and assoicate it with the
        // constructed filter from the inner object.
        } else if let Some((next_key, next_predicate)) = top_level_arg_value_iter.next() {
            match next_key {
                "and" | "or" => {
                    return parse_binary_logical_operator(
                        next_key,
                        next_predicate.clone(),
                        entity_type,
                        schema,
                        top_level_arg_value_iter,
                        &mut Some(filter),
                    )
                }
                other => {
                    let next_filter = parse_arg_pred_pair(
                        other,
                        next_predicate.clone(),
                        entity_type,
                        schema,
                        prior_filter,
                        top_level_arg_value_iter,
                    )?;
                    let final_filter = match key {
                        "and" => FilterType::LogicOp(LogicOp::And(
                            Box::new(filter),
                            Box::new(next_filter),
                        )),
                        "or" => FilterType::LogicOp(LogicOp::Or(
                            Box::new(filter),
                            Box::new(next_filter),
                        )),
                        _ => unreachable!(),
                    };
                    return Ok(final_filter);
                }
            }
        } else {
            return Err(GraphqlError::MissingPartnerForBinaryLogicalOperator);
        }
    } else {
        Err(GraphqlError::UnsupportedValueType(predicate.to_string()))
    }
}

/// Parse a value from the parsed GraphQL document into a `ParsedValue` for use in the indexer.
///
/// Value types from the parsed GraphQL query should be turned into `ParsedValue`
/// instances so that they can be properly formatted for transformation into SQL queries.
fn parse_value<'a>(value: &Value<'a, &'a str>) -> Result<ParsedValue, GraphqlError> {
    match value {
        Value::BigInt(bn) => Ok(ParsedValue::BigNumber(bn.as_u128())),
        Value::Boolean(b) => Ok(ParsedValue::Boolean(*b)),
        Value::Int(n) => Ok(ParsedValue::Number(n.as_u64())),
        Value::String(s) => Ok(ParsedValue::String(s.clone())),
        _ => Err(GraphqlError::UnsupportedValueType(value.to_string())),
    }
}

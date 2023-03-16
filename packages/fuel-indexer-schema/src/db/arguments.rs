use super::{graphql::GraphqlError, tables::Schema};

use fuel_indexer_database::DbType;
use graphql_parser::query::Value;
use std::{collections::BTreeMap, fmt};

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
pub enum Filter {
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
    And(Box<Filter>, Box<Filter>),
    Or(Box<Filter>, Box<Filter>),
    Not(Box<Filter>),
}

impl Filter {
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
                    // NOT cases are handled elsewhere
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

/// Negate a filter into its inverse or opposite filter.
///
/// Each filter should have a inverse type when negated in order to minimize
/// disruption to the user. When adding a new filter type, special consideration
/// should be given as to if and how it can be represented in the inverse.
trait Negatable {
    fn negate(&self) -> Result<Filter, GraphqlError>;
}

impl Negatable for Filter {
    fn negate(&self) -> Result<Filter, GraphqlError> {
        match self {
            Filter::IdSelection(_) => Err(GraphqlError::UnsupportedNegation(
                "ID selection".to_string(),
            )),
            Filter::Comparison(c) => match c {
                Comparison::Between(field, val1, val2) => {
                    Ok(Filter::LogicOp(LogicOp::And(
                        Box::new(Filter::Comparison(Comparison::Less(
                            field.clone(),
                            val1.clone(),
                        ))),
                        Box::new(Filter::Comparison(Comparison::Greater(
                            field.clone(),
                            val2.clone(),
                        ))),
                    )))
                }
                Comparison::Greater(field, val) => Ok(Filter::Comparison(
                    Comparison::LessEqual(field.clone(), val.clone()),
                )),
                Comparison::GreaterEqual(field, val) => Ok(Filter::Comparison(
                    Comparison::Less(field.clone(), val.clone()),
                )),
                Comparison::Less(field, val) => Ok(Filter::Comparison(
                    Comparison::GreaterEqual(field.clone(), val.clone()),
                )),
                Comparison::LessEqual(field, val) => Ok(Filter::Comparison(
                    Comparison::Greater(field.clone(), val.clone()),
                )),
                Comparison::Equals(field, val) => Ok(Filter::Comparison(
                    Comparison::NotEquals(field.clone(), val.clone()),
                )),
                Comparison::NotEquals(field, val) => Ok(Filter::Comparison(
                    Comparison::Equals(field.clone(), val.clone()),
                )),
            },
            Filter::Membership(mf) => match mf {
                Membership::In(field, element_list) => Ok(Filter::Membership(
                    Membership::NotIn(field.clone(), element_list.clone()),
                )),
                Membership::NotIn(field, element_list) => Ok(Filter::Membership(
                    Membership::In(field.clone(), element_list.clone()),
                )),
            },
            Filter::NullValueCheck(nvc) => match nvc {
                NullValueCheck::NoNulls(column_list) => Ok(Filter::NullValueCheck(
                    NullValueCheck::OnlyNulls(column_list.clone()),
                )),
                NullValueCheck::OnlyNulls(column_list) => Ok(Filter::NullValueCheck(
                    NullValueCheck::NoNulls(column_list.clone()),
                )),
            },
            Filter::LogicOp(lo) => match lo {
                LogicOp::And(r1, r2) => Ok(Filter::LogicOp(LogicOp::And(
                    Box::new(r1.clone().negate()?),
                    Box::new(r2.clone().negate()?),
                ))),
                LogicOp::Or(r1, r2) => Ok(Filter::LogicOp(LogicOp::Or(
                    Box::new(r1.clone().negate()?),
                    Box::new(r2.clone().negate()?),
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
pub fn parse_arguments<'a>(
    entity_type: &String,
    arg: &str,
    value: Value<'a, &'a str>,
    schema: &Schema,
) -> Result<Filter, GraphqlError> {
    // We instantiate an Option<Filter> in order to keep track of the last
    // seen filter in the event that an AND/OR operator is used; if so, the
    // prior filter is associated with the inner filter of the logical operator.
    let mut prior_filter: Option<Filter> = None;

    match arg {
        "filter" => {
            if let Value::Object(obj) = value {
                parse_filter_object(obj, entity_type, schema, &mut prior_filter)
            } else {
                Err(GraphqlError::UnsupportedValueType(value.to_string()))
            }
        }
        "id" => Ok(Filter::IdSelection(parse_value(&value)?)),
        "order" => todo!(),
        "offset" => todo!(),
        "first" => todo!(),
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
    prior_filter: &mut Option<Filter>,
) -> Result<Filter, GraphqlError> {
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
    prior_filter: &mut Option<Filter>,
    top_level_arg_value_iter: &mut impl Iterator<Item = (&'a str, Value<'a, &'a str>)>,
) -> Result<Filter, GraphqlError> {
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
                Ok(Filter::NullValueCheck(NullValueCheck::NoNulls(column_list)))
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
        "not" => {
            if let Value::Object(inner_obj) = predicate {
                parse_filter_object(inner_obj, entity_type, schema, prior_filter)?
                    .negate()
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
                                        return Ok(Filter::Comparison(
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
                                return Ok(Filter::Comparison(Comparison::Equals(
                                    other.to_string(),
                                    parse_value(predicate)?,
                                )))
                            }
                            "gt" => {
                                return Ok(Filter::Comparison(Comparison::Greater(
                                    other.to_string(),
                                    parse_value(predicate)?,
                                )))
                            }
                            "gte" => {
                                return Ok(Filter::Comparison(Comparison::GreaterEqual(
                                    other.to_string(),
                                    parse_value(predicate)?,
                                )));
                            }
                            "lt" => {
                                return Ok(Filter::Comparison(Comparison::Less(
                                    other.to_string(),
                                    parse_value(predicate)?,
                                )))
                            }
                            "lte" => {
                                return Ok(Filter::Comparison(Comparison::LessEqual(
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
                                        return Ok(Filter::Membership(Membership::In(
                                            other.to_string(),
                                            elements,
                                        )));
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
    prior_filter: &mut Option<Filter>,
) -> Result<Filter, GraphqlError> {
    if let Value::Object(inner_obj) = predicate {
        // Construct the filter contained in the object value for the binary logical operator
        let filter = parse_filter_object(inner_obj, entity_type, schema, prior_filter)?;

        // If we've already constructed a filter prior to this, associate it with
        // the filter that was just parsed from the inner object.
        if let Some(prior_filter) = prior_filter {
            match key {
                "and" => Ok(Filter::LogicOp(LogicOp::And(
                    Box::new(prior_filter.clone()),
                    Box::new(filter),
                ))),
                "or" => Ok(Filter::LogicOp(LogicOp::Or(
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
                        "and" => Filter::LogicOp(LogicOp::And(
                            Box::new(filter),
                            Box::new(next_filter),
                        )),
                        "or" => Filter::LogicOp(LogicOp::Or(
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

use fuel_indexer_types::scalar::{Boolean, UID};
use sqlparser::ast as sql;

/// Represents `WHERE filter ORDER BY ASC | DSC` part of the SQL statement.
pub struct QueryFragment<T> {
    constraint: Filter<T>,
    order_by: Option<sql::OrderByExpr>,
}

/// Convert `QueryFragment` to `String`. `SELECT * from table_name` is lated
/// added by the Fuel indexer to generate the entire query.
impl<T> std::fmt::Display for QueryFragment<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.constraint)?;
        if let Some(ref order_by) = self.order_by {
            write!(f, " ORDER BY {}", order_by)?;
        }
        Ok(())
    }
}

/// Automatic lifting of `Filter` into `QueryFragment` leaving `ORDER BY`
/// unspecified.
impl<T> From<Filter<T>> for QueryFragment<T> {
    fn from(constraint: Filter<T>) -> Self {
        QueryFragment {
            constraint,
            order_by: None,
        }
    }
}

/// Represents a WHERE clause of the SQL statement. Multiple `Filter`s can be
/// joined with `and` and `or` and also ordered, at which point they become
/// `QueryFragment`s.
pub struct Filter<T> {
    constraint: sql::Expr,
    phantom: std::marker::PhantomData<T>,
}

impl<T> std::fmt::Display for Filter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.constraint)
    }
}

impl<T> Filter<T> {
    fn new(constraint: sql::Expr) -> Self {
        Self {
            constraint,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn and(self, right: Filter<T>) -> Filter<T> {
        let constraint = sql::Expr::BinaryOp {
            left: Box::new(self.constraint),
            op: sql::BinaryOperator::And,
            right: Box::new(right.constraint),
        };
        Filter {
            constraint,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn or(self, right: Filter<T>) -> Filter<T> {
        let constraint = sql::Expr::BinaryOp {
            left: Box::new(self.constraint),
            op: sql::BinaryOperator::Or,
            right: Box::new(right.constraint),
        };
        Filter {
            constraint,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn order_by_asc<F>(self, f: Field<T, F>) -> QueryFragment<T> {
        QueryFragment {
            constraint: self,
            order_by: Some(sql::OrderByExpr {
                expr: sql::Expr::Identifier(sql::Ident::new(f.field)),
                asc: Some(true),
                nulls_first: None,
            }),
        }
    }

    pub fn order_by_desc<F>(self, f: Field<T, F>) -> QueryFragment<T> {
        QueryFragment {
            constraint: self,
            order_by: Some(sql::OrderByExpr {
                expr: sql::Expr::Identifier(sql::Ident::new(f.field)),
                asc: Some(false),
                nulls_first: None,
            }),
        }
    }
}

/// A trait used to convert a value of scalar type into `sqlparser::ast::Value`.
/// That is, for injecting a value into the `sqlparser`'s representation which
/// we then use to generate a `QueryFragment`.
pub trait ToSQLValue
where
    Self: Sized,
{
    fn to_sql_value(self) -> sql::Value;
}

impl ToSQLValue for String {
    fn to_sql_value(self) -> sql::Value {
        sql::Value::SingleQuotedString(self)
    }
}

impl ToSQLValue for Boolean {
    fn to_sql_value(self) -> sql::Value {
        sql::Value::Boolean(self)
    }
}

impl ToSQLValue for UID {
    fn to_sql_value(self) -> sql::Value {
        sql::Value::SingleQuotedString(self.to_string())
    }
}

macro_rules! impl_bytes_to_sql_value {
    ($T:ident) => {
        impl ToSQLValue for fuel_indexer_types::scalar::$T {
            fn to_sql_value(self) -> sql::Value {
                unsafe {
                    sql::Value::SingleQuotedByteStringLiteral(
                        std::str::from_utf8_unchecked(self.as_ref()).to_string(),
                    )
                }
            }
        }
    };
}

impl_bytes_to_sql_value!(B256);
impl_bytes_to_sql_value!(Bytes32);
impl_bytes_to_sql_value!(Bytes8);
impl_bytes_to_sql_value!(Bytes4);
impl_bytes_to_sql_value!(Bytes);
impl_bytes_to_sql_value!(AssetId);
impl_bytes_to_sql_value!(Address);
impl_bytes_to_sql_value!(ContractId);
impl_bytes_to_sql_value!(MessageId);
impl_bytes_to_sql_value!(Nonce);
impl_bytes_to_sql_value!(Salt);

macro_rules! impl_number_to_sql_value {
    ($T:ident) => {
        impl ToSQLValue for fuel_indexer_types::scalar::$T {
            fn to_sql_value(self) -> sql::Value {
                sqlparser::test_utils::number(&self.to_string())
            }
        }
    };
}

impl_number_to_sql_value!(I128);
impl_number_to_sql_value!(U128);

impl_number_to_sql_value!(I64);
impl_number_to_sql_value!(U64);

impl_number_to_sql_value!(I32);
impl_number_to_sql_value!(U32);

impl_number_to_sql_value!(I8);
impl_number_to_sql_value!(U8);

impl_number_to_sql_value!(BlockHeight);

/// Captures the information necessary to represent `struct T { field: F }`.
/// It is then used to build a type-safe `Filter<T>`, e.g., `Filter<OrderId>`.
pub struct Field<T, F> {
    field: String,
    phantom: std::marker::PhantomData<(T, F)>,
}

impl<T, F> Field<T, F> {
    pub fn new(field: String) -> Self {
        Field {
            field,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<T, F: ToSQLValue> Field<T, F> {
    pub fn eq(self, val: F) -> Filter<T> {
        self.filter(sql::BinaryOperator::Eq, val)
    }

    pub fn ne(self, val: F) -> Filter<T> {
        self.filter(sql::BinaryOperator::NotEq, val)
    }

    pub fn gt(self, val: F) -> Filter<T> {
        self.filter(sql::BinaryOperator::Gt, val)
    }

    pub fn ge(self, val: F) -> Filter<T> {
        self.filter(sql::BinaryOperator::GtEq, val)
    }

    pub fn lt(self, val: F) -> Filter<T> {
        self.filter(sql::BinaryOperator::Lt, val)
    }

    pub fn le(self, val: F) -> Filter<T> {
        self.filter(sql::BinaryOperator::LtEq, val)
    }

    fn filter(self, op: sql::BinaryOperator, val: F) -> Filter<T> {
        let expr = sql::Expr::BinaryOp {
            left: Box::new(sql::Expr::Identifier(sql::Ident::new(self.field.clone()))),
            op,
            right: Box::new(sql::Expr::Value(val.to_sql_value())),
        };
        Filter::new(expr)
    }
}

/// Captures the information necessary to represent `struct T { field: Option<F> }`
/// which requires additional logic for dealing with NULL values. Like `Field<T, F>`,
/// it is used to build a type-safe `Filter<T>`.
pub struct OptionField<T, F> {
    field: String,
    phantom: std::marker::PhantomData<(T, F)>,
}

impl<T, F> OptionField<T, F> {
    pub fn new(field: String) -> Self {
        OptionField {
            field,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<T, F: ToSQLValue> OptionField<T, F> {
    pub fn eq(self, val: F) -> Filter<T> {
        self.filter(sql::BinaryOperator::Eq, val)
    }

    pub fn ne(self, val: F) -> Filter<T> {
        self.filter(sql::BinaryOperator::NotEq, val)
    }

    pub fn gt(self, val: F) -> Filter<T> {
        self.filter(sql::BinaryOperator::Gt, val)
    }

    pub fn ge(self, val: F) -> Filter<T> {
        self.filter(sql::BinaryOperator::GtEq, val)
    }

    pub fn lt(self, val: F) -> Filter<T> {
        self.filter(sql::BinaryOperator::Lt, val)
    }

    pub fn le(self, val: F) -> Filter<T> {
        self.filter(sql::BinaryOperator::LtEq, val)
    }

    pub fn is_null(self) -> Filter<T> {
        Filter::new(sql::Expr::IsNull(Box::new(sql::Expr::Identifier(
            sql::Ident::new(self.field),
        ))))
    }

    pub fn is_not_null(self) -> Filter<T> {
        Filter::new(sql::Expr::IsNotNull(Box::new(sql::Expr::Identifier(
            sql::Ident::new(self.field),
        ))))
    }

    // Helper function that unwraps the Option converting None to NULL.
    fn filter(self, op: sql::BinaryOperator, val: F) -> Filter<T> {
        let expr = sql::Expr::BinaryOp {
            left: Box::new(sql::Expr::Identifier(sql::Ident::new(self.field))),
            op,
            right: Box::new(sql::Expr::Value(val.to_sql_value())),
        };
        Filter::new(expr)
    }
}

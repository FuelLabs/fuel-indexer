use fuel_indexer_types::scalar::{Boolean, UID};
use sqlparser::ast as sql;

/// Represents a filter that returns a single results.
pub struct SingleFilter<T> {
    filter: String,
    phantom: std::marker::PhantomData<T>,
}

impl<T> std::fmt::Display for SingleFilter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} LIMIT 1", self.filter)?;
        Ok(())
    }
}
/// Represents a filter with a an optional LIMIT clause that returns many
/// results.
pub struct ManyFilter<T> {
    filter: String,
    limit: Option<usize>,
    phantom: std::marker::PhantomData<T>,
}

impl<T> ManyFilter<T> {
    pub fn limit(&self) -> Option<usize> {
        self.limit
    }
}

impl<T> std::fmt::Display for ManyFilter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.filter)?;
        if let Some(limit) = self.limit {
            write!(f, " LIMIT {limit}")?;
        }
        Ok(())
    }
}

/// Represents `filter` and `order_by` parts of the `SELECT object from {table}
/// WHERE {filter} {order_by}` statement that is assembled by the indexer to
/// fetch an object from the database. The table name is not available to the
/// plugin and thus only a part of the statment is generated there. The indexer
/// maps the TYPE_ID to the tale name and assemles the full statemnt.
pub struct OrderedFilter<T> {
    filter: Filter<T>,
    order_by: sql::OrderByExpr,
}

impl<T> OrderedFilter<T> {
    pub fn asc(mut self) -> Self {
        self.order_by.asc = Some(true);
        self
    }

    pub fn desc(mut self) -> Self {
        self.order_by.asc = Some(false);
        self
    }

    pub fn limit(self, limit: usize) -> ManyFilter<T> {
        ManyFilter {
            filter: self.to_string(),
            limit: Some(limit),
            phantom: std::marker::PhantomData,
        }
    }
}

/// Convert `OrderedFilter` to `String`. `SELECT * from table_name` is later
/// added by the Fuel indexer to generate the entire query.
impl<T> std::fmt::Display for OrderedFilter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ORDER BY {}", self.filter, self.order_by)?;
        Ok(())
    }
}

// Conversions between different filter structs.

impl<T> From<Filter<T>> for SingleFilter<T> {
    fn from(filter: Filter<T>) -> SingleFilter<T> {
        SingleFilter {
            filter: filter.to_string(),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<T> From<OrderedFilter<T>> for SingleFilter<T> {
    fn from(filter: OrderedFilter<T>) -> SingleFilter<T> {
        SingleFilter {
            filter: filter.to_string(),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<T> From<Filter<T>> for ManyFilter<T> {
    fn from(filter: Filter<T>) -> ManyFilter<T> {
        ManyFilter {
            filter: filter.to_string(),
            limit: None,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<T> From<OrderedFilter<T>> for ManyFilter<T> {
    fn from(filter: OrderedFilter<T>) -> ManyFilter<T> {
        ManyFilter {
            filter: filter.to_string(),
            limit: None,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<T> From<SingleFilter<T>> for ManyFilter<T> {
    fn from(filter: SingleFilter<T>) -> ManyFilter<T> {
        ManyFilter {
            filter: filter.filter,
            limit: Some(1),
            phantom: std::marker::PhantomData,
        }
    }
}

/// Represents a WHERE clause of the SQL statement. Multiple `Filter`s can be
/// joined with `and` and `or` and also ordered, at which point they become
/// `OrderedFilter`s.
pub struct Filter<T> {
    filter: sql::Expr,
    phantom: std::marker::PhantomData<T>,
}

impl<T> std::fmt::Display for Filter<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.filter)
    }
}

impl<T> Filter<T> {
    fn new(filter: sql::Expr) -> Self {
        Self {
            filter,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn and(self, right: Filter<T>) -> Filter<T> {
        let filter = sql::Expr::BinaryOp {
            left: Box::new(self.filter),
            op: sql::BinaryOperator::And,
            right: Box::new(right.filter),
        };
        Filter {
            filter,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn or(self, right: Filter<T>) -> Filter<T> {
        let filter = sql::Expr::BinaryOp {
            left: Box::new(self.filter),
            op: sql::BinaryOperator::Or,
            right: Box::new(right.filter),
        };
        Filter {
            filter,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn order_by<F>(self, f: Field<T, F>) -> OrderedFilter<T> {
        OrderedFilter {
            filter: self,
            order_by: sql::OrderByExpr {
                expr: sql::Expr::Identifier(sql::Ident::new(f.field)),
                asc: None,
                nulls_first: None,
            },
        }
    }

    pub fn limit(self, limit: usize) -> ManyFilter<T> {
        ManyFilter {
            filter: self.to_string(),
            limit: Some(limit),
            phantom: std::marker::PhantomData,
        }
    }
}

/// A trait used to convert a value of scalar type into `sqlparser::ast::Value`.
/// That is, for injecting a value into the `sqlparser`'s representation which
/// we then use to generate a `OrderedFilter`.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_query_generation() {
        struct MyStruct {}

        fn my_field() -> Field<MyStruct, fuel_indexer_types::scalar::I32> {
            Field {
                field: "my_field".to_string(),
                phantom: std::marker::PhantomData,
            }
        }

        let f: Filter<MyStruct> = my_field().gt(7);
        assert_eq!(&f.to_string(), "my_field > 7");

        let f: OrderedFilter<MyStruct> = my_field().gt(7).order_by(my_field()).asc();
        assert_eq!(&f.to_string(), "my_field > 7 ORDER BY my_field ASC");

        // Converting to SingleFilter imposes a LIMIT 1
        let sf: SingleFilter<MyStruct> =
            my_field().gt(7).order_by(my_field()).asc().into();
        assert_eq!(
            &sf.to_string(),
            "my_field > 7 ORDER BY my_field ASC LIMIT 1"
        );

        // SingleFilter converted to ManyFilter retains the LIMIT 1
        let mf: ManyFilter<MyStruct> = sf.into();
        assert_eq!(
            &mf.to_string(),
            "my_field > 7 ORDER BY my_field ASC LIMIT 1"
        );

        // Converting to ManyFilter does not impose a LIMIT
        let mf: ManyFilter<MyStruct> =
            my_field().gt(7).order_by(my_field()).desc().into();
        assert_eq!(&mf.to_string(), "my_field > 7 ORDER BY my_field DESC");
    }
}

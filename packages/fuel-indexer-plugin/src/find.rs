use fuel_indexer_types::scalar::{Boolean, Bytes, B256, UID};
use sqlparser::ast as sql;

pub struct Query<T> {
    constraint: Constraint<T>,
    order_by: Option<sql::OrderByExpr>,
}

impl<T> std::fmt::Display for Query<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.constraint)?;
        if let Some(ref order_by) = self.order_by {
            write!(f, " ORDER BY {}", order_by)?;
        }
        Ok(())
    }
}

impl<T> From<Constraint<T>> for Query<T> {
    fn from(constraint: Constraint<T>) -> Self {
        Query {
            constraint,
            order_by: None,
        }
    }
}

pub struct Constraint<T> {
    constraint: sql::Expr,
    phantom: std::marker::PhantomData<T>,
}

impl<T> std::fmt::Display for Constraint<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.constraint)
    }
}

impl<T> Constraint<T> {
    fn new(constraint: sql::Expr) -> Self {
        Self {
            constraint,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn and(self, right: Constraint<T>) -> Constraint<T> {
        let constraint = sql::Expr::BinaryOp {
            left: Box::new(self.constraint),
            op: sql::BinaryOperator::And,
            right: Box::new(right.constraint),
        };
        Constraint {
            constraint,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn or(self, right: Constraint<T>) -> Constraint<T> {
        let constraint = sql::Expr::BinaryOp {
            left: Box::new(self.constraint),
            op: sql::BinaryOperator::Or,
            right: Box::new(right.constraint),
        };
        Constraint {
            constraint,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn order_by_asc<F>(self, f: Field<T, F>) -> Query<T> {
        Query {
            constraint: self,
            order_by: Some(sql::OrderByExpr {
                expr: sql::Expr::Identifier(sql::Ident::new(f.field)),
                asc: Some(true),
                nulls_first: None,
            }),
        }
    }

    pub fn order_by_desc<F>(self, f: Field<T, F>) -> Query<T> {
        Query {
            constraint: self,
            order_by: Some(sql::OrderByExpr {
                expr: sql::Expr::Identifier(sql::Ident::new(f.field)),
                asc: Some(false),
                nulls_first: None,
            }),
        }
    }
}

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

impl ToSQLValue for B256 {
    fn to_sql_value(self) -> sql::Value {
        unsafe {
            sql::Value::SingleQuotedByteStringLiteral(
                std::str::from_utf8_unchecked(&self).to_string(),
            )
        }
    }
}

impl ToSQLValue for Bytes {
    fn to_sql_value(self) -> sql::Value {
        unsafe {
            sql::Value::SingleQuotedByteStringLiteral(
                std::str::from_utf8_unchecked(&self).to_string(),
            )
        }
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

impl<T, F: ToSQLValue> Field<T, F> {
    pub fn eq(self, val: F) -> Constraint<T> {
        self.constraint(sql::BinaryOperator::Eq, val)
    }

    pub fn ne(self, val: F) -> Constraint<T> {
        self.constraint(sql::BinaryOperator::NotEq, val)
    }

    pub fn gt(self, val: F) -> Constraint<T> {
        self.constraint(sql::BinaryOperator::Gt, val)
    }

    pub fn ge(self, val: F) -> Constraint<T> {
        self.constraint(sql::BinaryOperator::GtEq, val)
    }

    pub fn lt(self, val: F) -> Constraint<T> {
        self.constraint(sql::BinaryOperator::Lt, val)
    }

    pub fn le(self, val: F) -> Constraint<T> {
        self.constraint(sql::BinaryOperator::LtEq, val)
    }

    fn constraint(self, op: sql::BinaryOperator, val: F) -> Constraint<T> {
        let expr = sql::Expr::BinaryOp {
            left: Box::new(sql::Expr::Identifier(sql::Ident::new(self.field.clone()))),
            op,
            right: Box::new(sql::Expr::Value(val.to_sql_value())),
        };
        Constraint::new(expr)
    }
}

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
    pub fn eq(self, val: F) -> Constraint<T> {
        self.constraint(sql::BinaryOperator::Eq, val)
    }

    pub fn ne(self, val: F) -> Constraint<T> {
        self.constraint(sql::BinaryOperator::NotEq, val)
    }

    pub fn gt(self, val: F) -> Constraint<T> {
        self.constraint(sql::BinaryOperator::Gt, val)
    }

    pub fn ge(self, val: F) -> Constraint<T> {
        self.constraint(sql::BinaryOperator::GtEq, val)
    }

    pub fn lt(self, val: F) -> Constraint<T> {
        self.constraint(sql::BinaryOperator::Lt, val)
    }

    pub fn le(self, val: F) -> Constraint<T> {
        self.constraint(sql::BinaryOperator::LtEq, val)
    }

    pub fn is_null(self) -> Constraint<T> {
        Constraint::new(sql::Expr::IsNull(Box::new(sql::Expr::Identifier(
            sql::Ident::new(self.field),
        ))))
    }

    pub fn is_not_null(self) -> Constraint<T> {
        Constraint::new(sql::Expr::IsNotNull(Box::new(sql::Expr::Identifier(
            sql::Ident::new(self.field),
        ))))
    }

    // Helper function that unwraps the Option converting None to NULL.
    fn constraint(self, op: sql::BinaryOperator, val: F) -> Constraint<T> {
        let expr = sql::Expr::BinaryOp {
            left: Box::new(sql::Expr::Identifier(sql::Ident::new(self.field))),
            op,
            right: Box::new(sql::Expr::Value(val.to_sql_value())),
        };
        Constraint::new(expr)
    }
}

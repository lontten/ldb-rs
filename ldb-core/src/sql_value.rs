//! SQL 参数值与 `IntoSqlValue` trait。

use std::fmt;

/// 绑定到 SQL 的参数值。
#[derive(Debug, Clone, PartialEq)]
pub enum SqlValue {
    Null,
    Bool(bool),
    I64(i64),
    U64(u64),
    F64(f64),
    String(String),
    Bytes(Vec<u8>),
    DateTime(chrono::DateTime<chrono::Utc>),
    Uuid(uuid::Uuid),
}

impl SqlValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            SqlValue::String(s) => Some(s),
            _ => None,
        }
    }
}

impl fmt::Display for SqlValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SqlValue::Null => write!(f, "NULL"),
            SqlValue::Bool(v) => write!(f, "{v}"),
            SqlValue::I64(v) => write!(f, "{v}"),
            SqlValue::U64(v) => write!(f, "{v}"),
            SqlValue::F64(v) => write!(f, "{v}"),
            SqlValue::String(v) => write!(f, "{v}"),
            SqlValue::Bytes(v) => write!(f, "{v:?}"),
            SqlValue::DateTime(v) => write!(f, "{v}"),
            SqlValue::Uuid(v) => write!(f, "{v}"),
        }
    }
}

/// 将 Rust 值转为 SQL 绑定参数。
pub trait IntoSqlValue {
    fn into_sql_value(self) -> SqlValue;
}

impl IntoSqlValue for SqlValue {
    fn into_sql_value(self) -> SqlValue {
        self
    }
}

impl IntoSqlValue for bool {
    fn into_sql_value(self) -> SqlValue {
        SqlValue::Bool(self)
    }
}

impl IntoSqlValue for i8 {
    fn into_sql_value(self) -> SqlValue {
        SqlValue::I64(self as i64)
    }
}

impl IntoSqlValue for i16 {
    fn into_sql_value(self) -> SqlValue {
        SqlValue::I64(self as i64)
    }
}

impl IntoSqlValue for i32 {
    fn into_sql_value(self) -> SqlValue {
        SqlValue::I64(self as i64)
    }
}

impl IntoSqlValue for i64 {
    fn into_sql_value(self) -> SqlValue {
        SqlValue::I64(self)
    }
}

impl IntoSqlValue for u8 {
    fn into_sql_value(self) -> SqlValue {
        SqlValue::U64(self as u64)
    }
}

impl IntoSqlValue for u16 {
    fn into_sql_value(self) -> SqlValue {
        SqlValue::U64(self as u64)
    }
}

impl IntoSqlValue for u32 {
    fn into_sql_value(self) -> SqlValue {
        SqlValue::U64(self as u64)
    }
}

impl IntoSqlValue for u64 {
    fn into_sql_value(self) -> SqlValue {
        SqlValue::U64(self)
    }
}

impl IntoSqlValue for f64 {
    fn into_sql_value(self) -> SqlValue {
        SqlValue::F64(self)
    }
}

impl IntoSqlValue for &str {
    fn into_sql_value(self) -> SqlValue {
        SqlValue::String(self.to_string())
    }
}

impl IntoSqlValue for String {
    fn into_sql_value(self) -> SqlValue {
        SqlValue::String(self)
    }
}

impl IntoSqlValue for Vec<u8> {
    fn into_sql_value(self) -> SqlValue {
        SqlValue::Bytes(self)
    }
}

impl IntoSqlValue for chrono::DateTime<chrono::Utc> {
    fn into_sql_value(self) -> SqlValue {
        SqlValue::DateTime(self)
    }
}

impl IntoSqlValue for uuid::Uuid {
    fn into_sql_value(self) -> SqlValue {
        SqlValue::Uuid(self)
    }
}

impl<T: IntoSqlValue> IntoSqlValue for Option<T> {
    fn into_sql_value(self) -> SqlValue {
        match self {
            Some(v) => v.into_sql_value(),
            None => SqlValue::Null,
        }
    }
}

pub fn sql_value_to_string(value: &SqlValue) -> String {
    match value {
        SqlValue::Null => "NULL".to_string(),
        SqlValue::Bool(v) => v.to_string(),
        SqlValue::I64(v) => v.to_string(),
        SqlValue::U64(v) => v.to_string(),
        SqlValue::F64(v) => v.to_string(),
        SqlValue::String(v) => v.clone(),
        SqlValue::Bytes(v) => format!("{v:?}"),
        SqlValue::DateTime(v) => v.to_string(),
        SqlValue::Uuid(v) => v.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option_none_is_null() {
        let v: Option<i64> = None;
        assert_eq!(v.into_sql_value(), SqlValue::Null);
    }

    #[test]
    fn string_conversion() {
        assert_eq!("tom".into_sql_value(), SqlValue::String("tom".into()));
    }

    #[test]
    fn numeric_conversions() {
        assert_eq!(1i8.into_sql_value(), SqlValue::I64(1));
        assert_eq!(2i16.into_sql_value(), SqlValue::I64(2));
        assert_eq!(3i32.into_sql_value(), SqlValue::I64(3));
        assert_eq!(4u8.into_sql_value(), SqlValue::U64(4));
        assert_eq!(5u16.into_sql_value(), SqlValue::U64(5));
        assert_eq!(6u32.into_sql_value(), SqlValue::U64(6));
        assert_eq!(7u64.into_sql_value(), SqlValue::U64(7));
        assert_eq!(1.5f64.into_sql_value(), SqlValue::F64(1.5));
        assert_eq!(true.into_sql_value(), SqlValue::Bool(true));
    }

    #[test]
    fn display_and_string_helpers() {
        assert_eq!(SqlValue::Null.to_string(), "NULL");
        assert_eq!(SqlValue::I64(3).to_string(), "3");
        assert_eq!(SqlValue::String("a".into()).as_str(), Some("a"));
        assert_eq!(sql_value_to_string(&SqlValue::Bool(false)), "false");
        assert_eq!(sql_value_to_string(&SqlValue::I64(1)), "1");
        assert_eq!(sql_value_to_string(&SqlValue::Null), "NULL");
    }
}

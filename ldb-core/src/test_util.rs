//! 测试用模型与 mock 引擎（单元测试）。

#![cfg(any(test, feature = "test-util", feature = "integration"))]

use crate::error::LdbError;
use crate::model::{ColumnMeta, LdbModel, TableConf};
use crate::sql_value::SqlValue;

/// 手写 `LdbModel` 实现，供单元测试与集成测试使用。
#[derive(Debug, Clone, Default)]
pub struct TestUser {
    pub id: Option<i64>,
    pub name: Option<String>,
    pub age: Option<i32>,
}

static TEST_USER_TABLE: TableConf = TableConf {
    table_name: "t_user",
    primary_key_column_name_list: &["id"],
    auto_column: Some("id"),
};

static TEST_USER_COLUMNS: [ColumnMeta; 3] = [
    ColumnMeta {
        field_name: "id",
        column_name: "id",
    },
    ColumnMeta {
        field_name: "name",
        column_name: "name",
    },
    ColumnMeta {
        field_name: "age",
        column_name: "age",
    },
];

impl LdbModel for TestUser {
    fn table_conf() -> &'static TableConf {
        &TEST_USER_TABLE
    }

    fn column_meta_list() -> &'static [ColumnMeta] {
        &TEST_USER_COLUMNS
    }

    fn field_sql_value(&self, field_name: &str) -> Option<SqlValue> {
        match field_name {
            "id" => self.id.map(SqlValue::I64),
            "name" => self.name.as_ref().map(|v| SqlValue::String(v.clone())),
            "age" => self.age.map(|v| SqlValue::I64(v as i64)),
            _ => None,
        }
    }

    fn set_field_sql_value(&mut self, field_name: &str, value: SqlValue) -> Result<(), LdbError> {
        match field_name {
            "id" => match value {
                SqlValue::Null => self.id = None,
                SqlValue::I64(n) => self.id = Some(n),
                _ => {
                    return Err(LdbError::ModelMapping("字段 `id` 类型不匹配".into()));
                }
            },
            "name" => match value {
                SqlValue::Null => self.name = None,
                SqlValue::String(s) => self.name = Some(s),
                _ => {
                    return Err(LdbError::ModelMapping("字段 `name` 类型不匹配".into()));
                }
            },
            "age" => match value {
                SqlValue::Null => self.age = None,
                SqlValue::I64(n) => self.age = Some(n as i32),
                _ => {
                    return Err(LdbError::ModelMapping("字段 `age` 类型不匹配".into()));
                }
            },
            _ => {}
        }
        Ok(())
    }
}

/// 条件模型。
#[derive(Debug, Clone, Default)]
pub struct TestUserWhere {
    pub name: Option<String>,
    pub age: Option<i32>,
}

impl LdbModel for TestUserWhere {
    fn table_conf() -> &'static TableConf {
        &TEST_USER_TABLE
    }

    fn column_meta_list() -> &'static [ColumnMeta] {
        &TEST_USER_COLUMNS
    }

    fn field_sql_value(&self, field_name: &str) -> Option<SqlValue> {
        match field_name {
            "name" => self.name.as_ref().map(|v| SqlValue::String(v.clone())),
            "age" => self.age.map(|v| SqlValue::I64(v as i64)),
            _ => None,
        }
    }

    fn set_field_sql_value(&mut self, field_name: &str, value: SqlValue) -> Result<(), LdbError> {
        match field_name {
            "name" => match value {
                SqlValue::Null => self.name = None,
                SqlValue::String(s) => self.name = Some(s),
                _ => {
                    return Err(LdbError::ModelMapping("字段 `name` 类型不匹配".into()));
                }
            },
            "age" => match value {
                SqlValue::Null => self.age = None,
                SqlValue::I64(n) => self.age = Some(n as i32),
                _ => {
                    return Err(LdbError::ModelMapping("字段 `age` 类型不匹配".into()));
                }
            },
            _ => {}
        }
        Ok(())
    }
}

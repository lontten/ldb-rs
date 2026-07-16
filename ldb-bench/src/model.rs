//! 基准用 ldb 模型（扩展列）。

use ldb_core::error::LdbError;
use ldb_core::model::{ColumnMeta, LdbModel, TableConf};
use ldb_core::sql_value::SqlValue;

/// 与 `t_user` 基准表对齐的模型。
#[derive(Debug, Clone, Default)]
pub struct BenchUser {
    pub id: Option<i64>,
    pub name: Option<String>,
    pub age: Option<i32>,
    pub status: Option<i16>,
    pub city: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

static TABLE: TableConf = TableConf {
    table_name: "t_user",
    primary_key_column_name_list: &["id"],
    auto_column: Some("id"),
    soft_delete_column: None,
};

static COLUMNS: [ColumnMeta; 6] = [
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
    ColumnMeta {
        field_name: "status",
        column_name: "status",
    },
    ColumnMeta {
        field_name: "city",
        column_name: "city",
    },
    ColumnMeta {
        field_name: "created_at",
        column_name: "created_at",
    },
];

impl LdbModel for BenchUser {
    fn table_conf() -> &'static TableConf {
        &TABLE
    }

    fn column_meta_list() -> &'static [ColumnMeta] {
        &COLUMNS
    }

    fn field_sql_value(&self, field_name: &str) -> Option<SqlValue> {
        match field_name {
            "id" => self.id.map(SqlValue::I64),
            "name" => self.name.as_ref().map(|v| SqlValue::String(v.clone())),
            "age" => self.age.map(|v| SqlValue::I64(v as i64)),
            "status" => self.status.map(|v| SqlValue::I64(v as i64)),
            "city" => self.city.as_ref().map(|v| SqlValue::String(v.clone())),
            "created_at" => self.created_at.map(SqlValue::DateTime),
            _ => None,
        }
    }

    fn set_field_sql_value(&mut self, field_name: &str, value: SqlValue) -> Result<(), LdbError> {
        match field_name {
            "id" => match value {
                SqlValue::Null => self.id = None,
                SqlValue::I64(n) => self.id = Some(n),
                _ => return Err(LdbError::ModelMapping("字段 `id` 类型不匹配".into())),
            },
            "name" => match value {
                SqlValue::Null => self.name = None,
                SqlValue::String(s) => self.name = Some(s),
                _ => return Err(LdbError::ModelMapping("字段 `name` 类型不匹配".into())),
            },
            "age" => match value {
                SqlValue::Null => self.age = None,
                SqlValue::I64(n) => self.age = Some(n as i32),
                _ => return Err(LdbError::ModelMapping("字段 `age` 类型不匹配".into())),
            },
            "status" => match value {
                SqlValue::Null => self.status = None,
                SqlValue::I64(n) => self.status = Some(n as i16),
                _ => return Err(LdbError::ModelMapping("字段 `status` 类型不匹配".into())),
            },
            "city" => match value {
                SqlValue::Null => self.city = None,
                SqlValue::String(s) => self.city = Some(s),
                _ => return Err(LdbError::ModelMapping("字段 `city` 类型不匹配".into())),
            },
            "created_at" => match value {
                SqlValue::Null => self.created_at = None,
                SqlValue::DateTime(t) => self.created_at = Some(t),
                _ => {
                    return Err(LdbError::ModelMapping(
                        "字段 `created_at` 类型不匹配".into(),
                    ));
                }
            },
            _ => {}
        }
        Ok(())
    }
}

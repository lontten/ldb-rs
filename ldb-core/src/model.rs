//! 模型元数据 trait 与表配置。

use crate::error::LdbError;
use crate::sql_value::SqlValue;

/// 表配置：表名、主键列、自增列。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableConf {
    pub table_name: &'static str,
    pub primary_key_column_name_list: &'static [&'static str],
    pub auto_column: Option<&'static str>,
}

/// 列元数据。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnMeta {
    pub field_name: &'static str,
    pub column_name: &'static str,
}

/// 由 `#[derive(LdbModel)]` 生成；用户通常不手动实现。
pub trait LdbModel: Send + Sync + Sized {
    fn table_conf() -> &'static TableConf;
    fn column_meta_list() -> &'static [ColumnMeta];

    /// 读取字段对应的 SQL 值（用于 insert/update/where model）。
    fn field_sql_value(&self, field_name: &str) -> Option<SqlValue>;

    /// 从 SQL 值写入字段（用于 select 行映射）。
    fn set_field_sql_value(&mut self, field_name: &str, value: SqlValue) -> Result<(), LdbError>;
}

/// 将列名 → 值映射填充为模型实例。
pub fn hydrate_model<T: LdbModel + Default>(
    column_value_list: &[(String, SqlValue)],
) -> Result<T, LdbError> {
    let mut model = T::default();
    for meta in T::column_meta_list() {
        if let Some((_, value)) = column_value_list
            .iter()
            .find(|(col, _)| col == meta.column_name)
        {
            model.set_field_sql_value(meta.field_name, value.clone())?;
        }
    }
    Ok(model)
}

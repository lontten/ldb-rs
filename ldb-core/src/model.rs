//! 模型元数据 trait 与表配置。

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
    fn field_sql_value(&self, field_name: &str) -> Option<crate::sql_value::SqlValue>;
}

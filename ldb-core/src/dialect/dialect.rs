//! SQL 方言抽象。

use std::borrow::Cow;

use crate::error::LdbError;

/// 按数据库类型处理占位符、标识符转义与 upsert 片段。
pub trait Dialect: Send + Sync {
    /// 返回该方言使用的占位符样式（如 `?` 或 `$1`）。
    fn placeholder_style(&self) -> PlaceholderStyle;

    /// 转义表名、列名等标识符。
    fn escape_identifier(&self, identifier: &str) -> String;

    /// 将逻辑 SQL 占位符改写为方言可执行形式。
    fn rewrite_sql<'a>(&self, query: &'a str) -> Cow<'a, str>;

    /// 生成 upsert 子句（MySQL `ON DUPLICATE KEY` / PG `ON CONFLICT`）。
    fn upsert_clause(
        &self,
        conflict_column_list: &[&str],
        update_column_list: &[&str],
        auto_column: Option<&str>,
    ) -> Result<String, LdbError>;
}

/// 占位符绑定风格。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceholderStyle {
    /// MySQL 风格：`?`
    QuestionMark,
    /// PostgreSQL 风格：`$1`, `$2`, …
    DollarNumbered,
}

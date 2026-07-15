//! MySQL 方言实现。

use std::borrow::Cow;

use crate::dialect::dialect::{Dialect, PlaceholderStyle};
use crate::error::LdbError;

/// MySQL 方言。
#[derive(Debug, Clone, Copy, Default)]
pub struct MysqlDialect;

impl Dialect for MysqlDialect {
    fn placeholder_style(&self) -> PlaceholderStyle {
        PlaceholderStyle::QuestionMark
    }

    fn escape_identifier(&self, identifier: &str) -> String {
        format!("`{identifier}`")
    }

    fn rewrite_sql<'a>(&self, query: &'a str) -> Cow<'a, str> {
        Cow::Borrowed(query)
    }

    fn upsert_clause(
        &self,
        _table: &str,
        conflict_column_list: &[&str],
    ) -> Result<String, LdbError> {
        if conflict_column_list.is_empty() {
            return Err(LdbError::SqlBuild("MySQL upsert 需要至少一个冲突列".into()));
        }
        Ok("ON DUPLICATE KEY UPDATE id = id".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_identifier_wraps_backticks() {
        let d = MysqlDialect;
        assert_eq!(d.escape_identifier("name"), "`name`");
    }

    #[test]
    fn rewrite_keeps_question_marks() {
        let d = MysqlDialect;
        let sql = d.rewrite_sql("SELECT * FROM t WHERE id = ?");
        assert!(matches!(sql, Cow::Borrowed(_)));
        assert_eq!(sql, "SELECT * FROM t WHERE id = ?");
    }

    #[test]
    fn upsert_requires_columns() {
        let d = MysqlDialect;
        assert!(d.upsert_clause("t", &[]).is_err());
        assert!(d.upsert_clause("t", &["name"]).is_ok());
    }
}

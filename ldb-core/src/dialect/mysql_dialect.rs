//! MySQL 方言实现。

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

    fn rewrite_exec(&self, query: &str, arg_list: &[String]) -> (String, Vec<String>) {
        (query.to_string(), arg_list.to_vec())
    }

    fn rewrite_query(&self, query: &str, arg_list: &[String]) -> (String, Vec<String>) {
        (query.to_string(), arg_list.to_vec())
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
        let (sql, args) = d.rewrite_exec("SELECT * FROM t WHERE id = ?", &["1".into()]);
        assert_eq!(sql, "SELECT * FROM t WHERE id = ?");
        assert_eq!(args, vec!["1"]);
    }

    #[test]
    fn upsert_requires_columns() {
        let d = MysqlDialect;
        assert!(d.upsert_clause("t", &[]).is_err());
        assert!(d.upsert_clause("t", &["name"]).is_ok());
    }
}

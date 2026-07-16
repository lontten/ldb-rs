//! MySQL 方言实现。

use std::borrow::Cow;

use crate::MysqlVersion;
use crate::dialect::dialect::{Dialect, PlaceholderStyle};
use crate::error::LdbError;

/// MySQL 方言。
#[derive(Debug, Clone, Copy, Default)]
pub struct MysqlDialect {
    pub(crate) version: MysqlVersion,
}

impl Dialect for MysqlDialect {
    fn placeholder_style(&self) -> PlaceholderStyle {
        PlaceholderStyle::QuestionMark
    }

    fn escape_identifier(&self, identifier: &str) -> String {
        format!("`{}`", identifier.replace('`', "``"))
    }

    fn rewrite_sql<'a>(&self, query: &'a str) -> Cow<'a, str> {
        Cow::Borrowed(query)
    }

    fn upsert_clause(
        &self,
        conflict_column_list: &[&str],
        update_column_list: &[&str],
        auto_column: Option<&str>,
    ) -> Result<String, LdbError> {
        if conflict_column_list.is_empty() {
            return Err(LdbError::SqlBuild("MySQL upsert 需要至少一个冲突列".into()));
        }
        let use_alias = matches!(
            self.version,
            MysqlVersion::Latest | MysqlVersion::V8_0_19 | MysqlVersion::V8_0_20
        );
        let mut set_part_list = update_column_list
            .iter()
            .map(|column| {
                let escaped = self.escape_identifier(column);
                if use_alias {
                    format!("{escaped} = new.{escaped}")
                } else {
                    format!("{escaped} = VALUES({escaped})")
                }
            })
            .collect::<Vec<_>>();
        if let Some(column) = auto_column {
            let escaped = self.escape_identifier(column);
            set_part_list.push(format!("{escaped} = LAST_INSERT_ID({escaped})"));
        }
        if set_part_list.is_empty() {
            return Err(LdbError::SqlBuild("MySQL upsert 没有可更新列".into()));
        }
        let alias = if use_alias { " AS new" } else { "" };
        Ok(format!(
            "{alias} ON DUPLICATE KEY UPDATE {}",
            set_part_list.join(", ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_identifier_wraps_backticks() {
        let d = MysqlDialect::default();
        assert_eq!(d.escape_identifier("name"), "`name`");
    }

    #[test]
    fn rewrite_keeps_question_marks() {
        let d = MysqlDialect::default();
        let sql = d.rewrite_sql("SELECT * FROM t WHERE id = ?");
        assert!(matches!(sql, Cow::Borrowed(_)));
        assert_eq!(sql, "SELECT * FROM t WHERE id = ?");
    }

    #[test]
    fn upsert_requires_columns() {
        let d = MysqlDialect::default();
        assert!(d.upsert_clause(&[], &["name"], None).is_err());
        assert!(d.upsert_clause(&["name"], &["name"], None).is_ok());
    }
}

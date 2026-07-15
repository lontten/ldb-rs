//! PostgreSQL 方言实现。

use std::borrow::Cow;

use crate::dialect::dialect::{Dialect, PlaceholderStyle};
use crate::error::LdbError;

/// PostgreSQL 方言。
#[derive(Debug, Clone, Copy, Default)]
pub struct PgDialect;

impl Dialect for PgDialect {
    fn placeholder_style(&self) -> PlaceholderStyle {
        PlaceholderStyle::DollarNumbered
    }

    fn escape_identifier(&self, identifier: &str) -> String {
        format!("\"{identifier}\"")
    }

    fn rewrite_sql<'a>(&self, query: &'a str) -> Cow<'a, str> {
        Cow::Owned(rewrite_placeholders(query))
    }

    fn upsert_clause(
        &self,
        table: &str,
        conflict_column_list: &[&str],
    ) -> Result<String, LdbError> {
        if conflict_column_list.is_empty() {
            return Err(LdbError::SqlBuild(
                "PostgreSQL upsert 需要至少一个冲突列".into(),
            ));
        }
        let cols = conflict_column_list
            .iter()
            .map(|c| self.escape_identifier(c))
            .collect::<Vec<_>>()
            .join(", ");
        let escaped_table = self.escape_identifier(table);
        Ok(format!(
            "ON CONFLICT ({cols}) DO UPDATE SET {escaped_table}.id = EXCLUDED.id"
        ))
    }
}

fn rewrite_placeholders(query: &str) -> String {
    let placeholder_count = query.as_bytes().iter().filter(|&&b| b == b'?').count();
    let mut out = String::with_capacity(query.len() + placeholder_count * 2);
    let mut arg_index = 1usize;
    for &byte in query.as_bytes() {
        if byte == b'?' {
            out.push('$');
            out.push_str(&arg_index.to_string());
            arg_index += 1;
        } else {
            out.push(byte as char);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_identifier_wraps_quotes() {
        let d = PgDialect;
        assert_eq!(d.escape_identifier("name"), "\"name\"");
    }

    #[test]
    fn rewrite_numbered_placeholders() {
        let d = PgDialect;
        let sql = d.rewrite_sql("SELECT * FROM t WHERE id = ? AND name = ?");
        assert_eq!(sql, "SELECT * FROM t WHERE id = $1 AND name = $2");
    }

    #[test]
    fn upsert_on_conflict() {
        let d = PgDialect;
        let clause = d.upsert_clause("t_user", &["name"]).unwrap();
        assert!(clause.contains("ON CONFLICT"));
        assert!(clause.contains("\"name\""));
    }
}

//! PostgreSQL 方言实现。

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

    fn rewrite_exec(&self, query: &str, arg_list: &[String]) -> (String, Vec<String>) {
        rewrite_placeholders(query, arg_list)
    }

    fn rewrite_query(&self, query: &str, arg_list: &[String]) -> (String, Vec<String>) {
        rewrite_placeholders(query, arg_list)
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

fn rewrite_placeholders(query: &str, arg_list: &[String]) -> (String, Vec<String>) {
    let mut out = String::with_capacity(query.len());
    let mut index = 0usize;
    let mut arg_index = 1usize;
    for ch in query.chars() {
        if ch == '?' {
            let _ = index;
            index += 1;
            out.push('$');
            out.push_str(&arg_index.to_string());
            arg_index += 1;
        } else {
            out.push(ch);
        }
    }
    (out, arg_list.to_vec())
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
        let (sql, args) = d.rewrite_query(
            "SELECT * FROM t WHERE id = ? AND name = ?",
            &["1".into(), "tom".into()],
        );
        assert_eq!(sql, "SELECT * FROM t WHERE id = $1 AND name = $2");
        assert_eq!(args.len(), 2);
    }

    #[test]
    fn upsert_on_conflict() {
        let d = PgDialect;
        let clause = d.upsert_clause("t_user", &["name"]).unwrap();
        assert!(clause.contains("ON CONFLICT"));
        assert!(clause.contains("\"name\""));
    }
}

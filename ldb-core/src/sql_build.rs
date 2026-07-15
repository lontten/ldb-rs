//! SQL 生成（纯逻辑，便于单元测试）。

use crate::PlaceholderStyle;
use crate::dialect::dialect::Dialect;
use crate::error::LdbError;
use crate::model::LdbModel;
use crate::on_conflict::OnConflict;
use crate::order::{Order, OrderBy};
use crate::sql_value::SqlValue;
use crate::where_builder::WhereBuilder;

/// Insert SQL 构建结果。
#[derive(Debug, Clone, PartialEq)]
pub struct BuiltSql {
    pub sql: String,
    pub arg_list: Vec<SqlValue>,
}

/// 构建 INSERT 语句。
pub fn build_insert<M: LdbModel>(
    table_name: &str,
    model: &M,
    on_conflict: Option<&OnConflict>,
    dialect: &dyn Dialect,
) -> Result<BuiltSql, LdbError> {
    let mut columns = vec![];
    let mut placeholders = vec![];
    let mut arg_list = vec![];
    for meta in M::column_meta_list() {
        if let Some(value) = model.field_sql_value(meta.field_name)
            && !matches!(value, SqlValue::Null)
        {
            columns.push(dialect.escape_identifier(meta.column_name));
            placeholders.push('?');
            arg_list.push(value);
        }
    }
    if columns.is_empty() {
        return Err(LdbError::SqlBuild("insert 无有效字段".into()));
    }
    let escaped_table = dialect.escape_identifier(table_name);
    let mut sql = format!(
        "INSERT INTO {escaped_table} ({}) VALUES ({})",
        columns.join(", "),
        placeholders
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(", ")
    );
    if let Some(conflict) = on_conflict {
        sql.push(' ');
        sql.push_str(&render_on_conflict(conflict, table_name, dialect)?);
    }
    Ok(BuiltSql { sql, arg_list })
}

fn render_on_conflict(
    conflict: &OnConflict,
    table: &str,
    dialect: &dyn Dialect,
) -> Result<String, LdbError> {
    match conflict {
        OnConflict::DoNothing => {
            if dialect.placeholder_style() == PlaceholderStyle::DollarNumbered {
                Ok("ON CONFLICT DO NOTHING".to_string())
            } else {
                Ok("ON DUPLICATE KEY UPDATE id = id".to_string())
            }
        }
        OnConflict::UpdateKey { column_name_list } => {
            let refs: Vec<&str> = column_name_list.iter().map(String::as_str).collect();
            dialect.upsert_clause(table, &refs)
        }
        OnConflict::UpdateAll => dialect.upsert_clause(table, &["id"]),
    }
}

/// 构建 UPDATE 语句（按 patch 字段 + 可选 extra set）。
pub fn build_update<M: LdbModel>(
    table_name: &str,
    patch: &M,
    where_builder: &WhereBuilder,
    extra_set_list: &[(String, SqlValue)],
) -> Result<BuiltSql, LdbError> {
    if where_builder.is_empty() {
        return Err(LdbError::WhereRequired);
    }
    let mut set_part_list = vec![];
    let mut arg_list = vec![];
    for meta in M::column_meta_list() {
        if let Some(value) = patch.field_sql_value(meta.field_name)
            && !matches!(value, SqlValue::Null)
        {
            set_part_list.push(format!("{} = ?", meta.column_name));
            arg_list.push(value);
        }
    }
    for (col, val) in extra_set_list {
        set_part_list.push(format!("{col} = ?"));
        arg_list.push(val.clone());
    }
    if set_part_list.is_empty() {
        return Err(LdbError::SqlBuild("update 无 SET 字段".into()));
    }
    let (where_sql, where_args) = where_builder.to_sql()?;
    let sql = format!(
        "UPDATE {table_name} SET {} WHERE {where_sql}",
        set_part_list.join(", ")
    );
    arg_list.extend(where_args);
    Ok(BuiltSql { sql, arg_list })
}

/// 构建 DELETE 语句。
pub fn build_delete(
    table_name: &str,
    where_builder: &WhereBuilder,
    allow_full_table: bool,
) -> Result<BuiltSql, LdbError> {
    if where_builder.is_empty() && !allow_full_table {
        return Err(LdbError::FullTableOpNotAllowed);
    }
    let (where_sql, arg_list) = where_builder.to_sql()?;
    let sql = if where_sql.is_empty() {
        format!("DELETE FROM {table_name}")
    } else {
        format!("DELETE FROM {table_name} WHERE {where_sql}")
    };
    Ok(BuiltSql { sql, arg_list })
}

/// SELECT 查询类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectKind {
    First,
    List,
    Has,
    Count,
}

/// 构建 SELECT 语句。
pub fn build_select<M: LdbModel>(
    table_name: &str,
    where_builder: &WhereBuilder,
    kind: SelectKind,
    order_by_list: &[OrderBy],
    limit: Option<u64>,
    offset: Option<u64>,
) -> Result<BuiltSql, LdbError> {
    if where_builder.is_empty() {
        return Err(LdbError::WhereRequired);
    }
    let (where_sql, arg_list) = where_builder.to_sql()?;
    let column_list = M::column_meta_list()
        .iter()
        .map(|m| m.column_name)
        .collect::<Vec<_>>()
        .join(", ");
    let select_cols = match kind {
        SelectKind::Count => "COUNT(*)".to_string(),
        SelectKind::Has => "1".to_string(),
        _ => column_list,
    };
    let mut sql = format!("SELECT {select_cols} FROM {table_name} WHERE {where_sql}");
    if !order_by_list.is_empty() && matches!(kind, SelectKind::First | SelectKind::List) {
        let order = order_by_list
            .iter()
            .map(|o| {
                let dir = match o.order {
                    Order::Asc => "ASC",
                    Order::Desc => "DESC",
                };
                format!("{} {dir}", o.column)
            })
            .collect::<Vec<_>>()
            .join(", ");
        sql.push_str(" ORDER BY ");
        sql.push_str(&order);
    }
    let effective_limit = match kind {
        SelectKind::First => Some(1),
        SelectKind::Has => Some(1),
        _ => limit,
    };
    if let Some(n) = effective_limit {
        sql.push_str(" LIMIT ");
        sql.push_str(&n.to_string());
    }
    if let Some(n) = offset
        && matches!(kind, SelectKind::List)
    {
        sql.push_str(" OFFSET ");
        sql.push_str(&n.to_string());
    }
    Ok(BuiltSql { sql, arg_list })
}

/// 将逻辑 SQL 按方言改写为可执行形式。
pub fn dialect_exec_sql<'a>(
    dialect: &dyn Dialect,
    built: &'a BuiltSql,
) -> std::borrow::Cow<'a, str> {
    dialect.rewrite_sql(&built.sql)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialect::mysql_dialect::MysqlDialect;
    use crate::test_util::TestUser;
    use crate::where_builder::w;

    #[test]
    fn build_insert_sql() {
        let user = TestUser {
            id: None,
            name: Some("tom".into()),
            age: Some(18),
        };
        let built = build_insert("t_user", &user, None, &MysqlDialect).unwrap();
        assert!(built.sql.starts_with("INSERT INTO `t_user`"));
        assert_eq!(built.arg_list.len(), 2);
    }

    #[test]
    fn build_delete_requires_where_or_allow() {
        let err = build_delete("t_user", &w(), false).unwrap_err();
        assert!(matches!(err, LdbError::FullTableOpNotAllowed));
    }

    #[test]
    fn build_update_with_where() {
        let patch = TestUser {
            id: None,
            name: Some("new".into()),
            age: None,
        };
        let built = build_update("t_user", &patch, &w().eq("id", 1), &[]).unwrap();
        assert!(built.sql.contains("UPDATE t_user SET"));
        assert!(built.sql.contains("WHERE"));
    }

    #[test]
    fn build_select_count() {
        let built = build_select::<TestUser>(
            "t_user",
            &w().gt("age", 0),
            SelectKind::Count,
            &[],
            None,
            None,
        )
        .unwrap();
        assert!(built.sql.contains("COUNT(*)"));
    }

    #[test]
    fn pg_on_conflict_do_nothing() {
        let user = TestUser {
            id: None,
            name: Some("x".into()),
            age: None,
        };
        let built = build_insert(
            "t_user",
            &user,
            Some(&OnConflict::DoNothing),
            &crate::dialect::pg_dialect::PgDialect,
        )
        .unwrap();
        assert!(built.sql.contains("ON CONFLICT DO NOTHING"));
    }

    #[test]
    fn insert_empty_fields_errors() {
        let user = TestUser::default();
        let err = build_insert("t_user", &user, None, &MysqlDialect).unwrap_err();
        assert!(matches!(err, LdbError::SqlBuild(_)));
    }

    #[test]
    fn dialect_exec_sql_rewrites_pg() {
        let built = BuiltSql {
            sql: "SELECT * FROM t WHERE id = ?".into(),
            arg_list: vec![SqlValue::I64(1)],
        };
        let sql = dialect_exec_sql(&crate::dialect::pg_dialect::PgDialect, &built);
        assert_eq!(sql, "SELECT * FROM t WHERE id = $1");
    }

    #[test]
    fn build_select_list_with_order_offset() {
        let built = build_select::<TestUser>(
            "t_user",
            &w().gt("id", 0),
            SelectKind::List,
            &[OrderBy {
                column: "id".into(),
                order: Order::Desc,
            }],
            Some(10),
            Some(5),
        )
        .unwrap();
        assert!(built.sql.contains("ORDER BY"));
        assert!(built.sql.contains("LIMIT"));
        assert!(built.sql.contains("OFFSET"));
    }
}

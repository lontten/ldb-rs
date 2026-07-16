//! WHERE 条件构建器。

use crate::dialect::dialect::Dialect;
use crate::error::LdbError;
use crate::sql_value::{IntoSqlValue, SqlValue};

/// 链式 WHERE 条件构建器。
#[derive(Debug, Clone, Default)]
pub struct WhereBuilder {
    node_list: Vec<WhereNode>,
}

#[derive(Debug, Clone)]
enum WhereNode {
    Group(WhereGroup),
}

#[derive(Debug, Clone)]
struct WhereGroup {
    negate: bool,
    item_list: Vec<WhereItem>,
}

#[derive(Debug, Clone)]
enum WhereItem {
    And(WhereBuilder),
    Or(WhereBuilder),
    Leaf(WhereLeaf),
}

#[derive(Debug, Clone)]
enum WhereLeaf {
    Eq {
        column: String,
        value: SqlValue,
    },
    NotEq {
        column: String,
        value: SqlValue,
    },
    InList {
        column: String,
        value_list: Vec<SqlValue>,
    },
    NotInList {
        column: String,
        value_list: Vec<SqlValue>,
    },
    Gt {
        column: String,
        value: SqlValue,
    },
    Gte {
        column: String,
        value: SqlValue,
    },
    Lt {
        column: String,
        value: SqlValue,
    },
    Lte {
        column: String,
        value: SqlValue,
    },
    Between {
        column: String,
        low: SqlValue,
        high: SqlValue,
    },
    IsNull {
        column: String,
    },
    IsNotNull {
        column: String,
    },
    Like {
        column: String,
        pattern: String,
    },
    LikeLeft {
        column: String,
        pattern: String,
    },
    LikeRight {
        column: String,
        pattern: String,
    },
    NotLike {
        column: String,
        pattern: String,
    },
    Native {
        sql: String,
        arg_list: Vec<SqlValue>,
    },
}

/// 创建空的 `WhereBuilder`。
pub fn w() -> WhereBuilder {
    WhereBuilder::default()
}

impl WhereBuilder {
    fn current_group_mut(&mut self) -> &mut WhereGroup {
        if self.node_list.is_empty() {
            self.node_list.push(WhereNode::Group(WhereGroup {
                negate: false,
                item_list: vec![],
            }));
        }
        match &mut self.node_list[0] {
            WhereNode::Group(g) => g,
        }
    }

    fn push_leaf(mut self, leaf: WhereLeaf) -> Self {
        self.current_group_mut()
            .item_list
            .push(WhereItem::Leaf(leaf));
        self
    }

    pub fn model<M: crate::model::LdbModel>(mut self, cond: &M) -> Self {
        for meta in M::column_meta_list() {
            if let Some(value) = cond.field_sql_value(meta.field_name)
                && !matches!(value, SqlValue::Null)
            {
                self = self.eq(meta.column_name, value);
            }
        }
        self
    }

    pub fn primary_key<M: crate::model::LdbModel>(self, id: impl IntoSqlValue) -> Self {
        match M::table_conf().primary_key_column_name_list {
            [column] => self.eq(column, id),
            _ => self,
        }
    }

    pub fn primary_key_model<M: crate::model::LdbModel>(mut self, model: &M) -> Self {
        for column in M::table_conf().primary_key_column_name_list {
            if let Some(meta) = M::column_meta_list()
                .iter()
                .find(|meta| meta.column_name == *column)
                && let Some(value) = model.field_sql_value(meta.field_name)
            {
                self = self.eq(column, value);
            }
        }
        self
    }

    pub fn filter_primary_key<M: crate::model::LdbModel>(mut self) -> Self {
        for column in M::table_conf().primary_key_column_name_list {
            self = self.is_not_null(column);
        }
        self
    }

    pub fn eq(self, column: &str, value: impl IntoSqlValue) -> Self {
        self.push_leaf(WhereLeaf::Eq {
            column: column.to_string(),
            value: value.into_sql_value(),
        })
    }

    pub fn eq_if(self, column: &str, value: impl IntoSqlValue, enabled: bool) -> Self {
        if enabled {
            self.eq(column, value)
        } else {
            self
        }
    }

    pub fn not_eq(self, column: &str, value: impl IntoSqlValue) -> Self {
        self.push_leaf(WhereLeaf::NotEq {
            column: column.to_string(),
            value: value.into_sql_value(),
        })
    }

    pub fn in_list(
        self,
        column: &str,
        values: impl IntoIterator<Item = impl IntoSqlValue>,
    ) -> Self {
        self.push_leaf(WhereLeaf::InList {
            column: column.to_string(),
            value_list: values.into_iter().map(|v| v.into_sql_value()).collect(),
        })
    }

    pub fn not_in_list(
        self,
        column: &str,
        values: impl IntoIterator<Item = impl IntoSqlValue>,
    ) -> Self {
        self.push_leaf(WhereLeaf::NotInList {
            column: column.to_string(),
            value_list: values.into_iter().map(|v| v.into_sql_value()).collect(),
        })
    }

    pub fn gt(self, column: &str, value: impl IntoSqlValue) -> Self {
        self.push_leaf(WhereLeaf::Gt {
            column: column.to_string(),
            value: value.into_sql_value(),
        })
    }

    pub fn gte(self, column: &str, value: impl IntoSqlValue) -> Self {
        self.push_leaf(WhereLeaf::Gte {
            column: column.to_string(),
            value: value.into_sql_value(),
        })
    }

    pub fn lt(self, column: &str, value: impl IntoSqlValue) -> Self {
        self.push_leaf(WhereLeaf::Lt {
            column: column.to_string(),
            value: value.into_sql_value(),
        })
    }

    pub fn lte(self, column: &str, value: impl IntoSqlValue) -> Self {
        self.push_leaf(WhereLeaf::Lte {
            column: column.to_string(),
            value: value.into_sql_value(),
        })
    }

    pub fn between(self, column: &str, low: impl IntoSqlValue, high: impl IntoSqlValue) -> Self {
        self.push_leaf(WhereLeaf::Between {
            column: column.to_string(),
            low: low.into_sql_value(),
            high: high.into_sql_value(),
        })
    }

    pub fn is_null(self, column: &str) -> Self {
        self.push_leaf(WhereLeaf::IsNull {
            column: column.to_string(),
        })
    }

    pub fn is_not_null(self, column: &str) -> Self {
        self.push_leaf(WhereLeaf::IsNotNull {
            column: column.to_string(),
        })
    }

    pub fn like(self, column: &str, pattern: &str) -> Self {
        self.push_leaf(WhereLeaf::Like {
            column: column.to_string(),
            pattern: pattern.to_string(),
        })
    }

    pub fn like_left(self, column: &str, pattern: &str) -> Self {
        self.push_leaf(WhereLeaf::LikeLeft {
            column: column.to_string(),
            pattern: pattern.to_string(),
        })
    }

    pub fn like_right(self, column: &str, pattern: &str) -> Self {
        self.push_leaf(WhereLeaf::LikeRight {
            column: column.to_string(),
            pattern: pattern.to_string(),
        })
    }

    pub fn not_like(self, column: &str, pattern: &str) -> Self {
        self.push_leaf(WhereLeaf::NotLike {
            column: column.to_string(),
            pattern: pattern.to_string(),
        })
    }

    pub fn and(mut self, other: WhereBuilder) -> Self {
        self.current_group_mut()
            .item_list
            .push(WhereItem::And(other));
        self
    }

    pub fn or(mut self, other: WhereBuilder) -> Self {
        self.current_group_mut()
            .item_list
            .push(WhereItem::Or(other));
        self
    }

    #[allow(clippy::should_implement_trait)]
    pub fn not(mut self) -> Self {
        if let Some(WhereNode::Group(g)) = self.node_list.first_mut() {
            g.negate = !g.negate;
        }
        self
    }

    pub fn native(self, sql: &str, arg_list: impl IntoIterator<Item = impl IntoSqlValue>) -> Self {
        self.push_leaf(WhereLeaf::Native {
            sql: sql.to_string(),
            arg_list: arg_list.into_iter().map(|v| v.into_sql_value()).collect(),
        })
    }

    pub fn native_if(
        self,
        enabled: bool,
        sql: &str,
        arg_list: impl IntoIterator<Item = impl IntoSqlValue>,
    ) -> Self {
        if enabled {
            self.native(sql, arg_list)
        } else {
            self
        }
    }

    /// 是否未设置任何条件。
    pub fn is_empty(&self) -> bool {
        self.node_list.is_empty()
            || self
                .node_list
                .iter()
                .all(|n| matches!(n, WhereNode::Group(g) if g.item_list.is_empty()))
    }

    /// 生成 `WHERE` 子句（不含 `WHERE` 关键字）与绑定参数。
    pub fn to_sql(&self) -> Result<(String, Vec<SqlValue>), LdbError> {
        self.render(None)
    }

    /// 使用数据库方言转义标识符后生成条件 SQL。
    pub fn to_sql_with_dialect(
        &self,
        dialect: &dyn Dialect,
    ) -> Result<(String, Vec<SqlValue>), LdbError> {
        self.render(Some(dialect))
    }

    fn render(&self, dialect: Option<&dyn Dialect>) -> Result<(String, Vec<SqlValue>), LdbError> {
        if self.is_empty() {
            return Ok((String::new(), vec![]));
        }
        let mut args = vec![];
        let mut sql = String::new();
        for node in &self.node_list {
            let WhereNode::Group(group) = node;
            render_group(group, &mut sql, &mut args, dialect)?;
        }
        Ok((sql, args))
    }
}

fn render_group(
    group: &WhereGroup,
    sql: &mut String,
    args: &mut Vec<SqlValue>,
    dialect: Option<&dyn Dialect>,
) -> Result<(), LdbError> {
    if group.item_list.is_empty() {
        return Ok(());
    }
    if group.negate {
        sql.push_str("NOT (");
    }
    for (i, item) in group.item_list.iter().enumerate() {
        if i > 0 {
            sql.push_str(match item {
                WhereItem::Or(_) => " OR ",
                _ => " AND ",
            });
        }
        render_item(item, sql, args, dialect)?;
    }
    if group.negate {
        sql.push(')');
    }
    Ok(())
}

fn render_item(
    item: &WhereItem,
    sql: &mut String,
    args: &mut Vec<SqlValue>,
    dialect: Option<&dyn Dialect>,
) -> Result<(), LdbError> {
    match item {
        WhereItem::And(inner) => {
            let (s, a) = inner.render(dialect)?;
            if s.is_empty() {
                return Ok(());
            }
            sql.push('(');
            sql.push_str(&s);
            sql.push(')');
            args.extend(a);
        }
        WhereItem::Or(inner) => {
            let (s, a) = inner.render(dialect)?;
            if s.is_empty() {
                return Ok(());
            }
            sql.push('(');
            sql.push_str(&s);
            sql.push(')');
            args.extend(a);
        }
        WhereItem::Leaf(leaf) => render_leaf(leaf, sql, args, dialect)?,
    }
    Ok(())
}

fn render_leaf(
    leaf: &WhereLeaf,
    sql: &mut String,
    args: &mut Vec<SqlValue>,
    dialect: Option<&dyn Dialect>,
) -> Result<(), LdbError> {
    let escaped = |column: &str| {
        dialect
            .map(|d| d.escape_identifier(column))
            .unwrap_or_else(|| column.to_string())
    };
    match leaf {
        WhereLeaf::Eq { column, value } => {
            sql.push_str(&escaped(column));
            sql.push_str(" = ?");
            args.push(value.clone());
        }
        WhereLeaf::NotEq { column, value } => {
            sql.push_str(&escaped(column));
            sql.push_str(" <> ?");
            args.push(value.clone());
        }
        WhereLeaf::InList { column, value_list } => {
            if value_list.is_empty() {
                return Err(LdbError::SqlBuild("IN 列表不能为空".into()));
            }
            sql.push_str(&escaped(column));
            sql.push_str(" IN (");
            for (i, _) in value_list.iter().enumerate() {
                if i > 0 {
                    sql.push_str(", ?");
                } else {
                    sql.push('?');
                }
            }
            sql.push(')');
            args.extend(value_list.clone());
        }
        WhereLeaf::NotInList { column, value_list } => {
            sql.push_str(&escaped(column));
            sql.push_str(" NOT IN (");
            for (i, _) in value_list.iter().enumerate() {
                if i > 0 {
                    sql.push_str(", ?");
                } else {
                    sql.push('?');
                }
            }
            sql.push(')');
            args.extend(value_list.clone());
        }
        WhereLeaf::Gt { column, value } => {
            sql.push_str(&escaped(column));
            sql.push_str(" > ?");
            args.push(value.clone());
        }
        WhereLeaf::Gte { column, value } => {
            sql.push_str(&escaped(column));
            sql.push_str(" >= ?");
            args.push(value.clone());
        }
        WhereLeaf::Lt { column, value } => {
            sql.push_str(&escaped(column));
            sql.push_str(" < ?");
            args.push(value.clone());
        }
        WhereLeaf::Lte { column, value } => {
            sql.push_str(&escaped(column));
            sql.push_str(" <= ?");
            args.push(value.clone());
        }
        WhereLeaf::Between { column, low, high } => {
            sql.push_str(&escaped(column));
            sql.push_str(" BETWEEN ? AND ?");
            args.push(low.clone());
            args.push(high.clone());
        }
        WhereLeaf::IsNull { column } => {
            sql.push_str(&escaped(column));
            sql.push_str(" IS NULL");
        }
        WhereLeaf::IsNotNull { column } => {
            sql.push_str(&escaped(column));
            sql.push_str(" IS NOT NULL");
        }
        WhereLeaf::Like { column, pattern } => {
            sql.push_str(&escaped(column));
            sql.push_str(" LIKE ?");
            args.push(SqlValue::String(pattern.clone()));
        }
        WhereLeaf::LikeLeft { column, pattern } => {
            sql.push_str(&escaped(column));
            sql.push_str(" LIKE ?");
            args.push(SqlValue::String(format!("%{pattern}")));
        }
        WhereLeaf::LikeRight { column, pattern } => {
            sql.push_str(&escaped(column));
            sql.push_str(" LIKE ?");
            args.push(SqlValue::String(format!("{pattern}%")));
        }
        WhereLeaf::NotLike { column, pattern } => {
            sql.push_str(&escaped(column));
            sql.push_str(" NOT LIKE ?");
            args.push(SqlValue::String(pattern.clone()));
        }
        WhereLeaf::Native {
            sql: frag,
            arg_list,
        } => {
            sql.push_str(frag);
            args.extend(arg_list.clone());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::TestUserWhere;

    #[test]
    fn eq_generates_placeholder() {
        let (sql, args) = w().eq("name", "tom").to_sql().unwrap();
        assert_eq!(sql, "name = ?");
        assert_eq!(args.len(), 1);
    }

    #[test]
    fn and_or_composition() {
        let (sql, args) = w().eq("age", 18).or(w().is_null("age")).to_sql().unwrap();
        assert_eq!(sql, "age = ? OR (age IS NULL)");
        assert_eq!(args.len(), 1);
    }

    #[test]
    fn in_list_placeholders() {
        let (sql, args) = w().in_list("id", [1i64, 2, 3]).to_sql().unwrap();
        assert_eq!(sql, "id IN (?, ?, ?)");
        assert_eq!(args.len(), 3);
    }

    #[test]
    fn empty_where() {
        let (sql, args) = w().to_sql().unwrap();
        assert!(sql.is_empty());
        assert!(args.is_empty());
    }

    #[test]
    fn comparison_operators() {
        let (sql, args) = w()
            .gt("age", 1)
            .and(w().gte("age", 2))
            .and(w().lt("age", 99))
            .and(w().lte("age", 100))
            .to_sql()
            .unwrap();
        assert!(sql.contains("age > ?"));
        assert!(sql.contains("age >= ?"));
        assert!(sql.contains("age < ?"));
        assert!(sql.contains("age <= ?"));
        assert_eq!(args.len(), 4);
    }

    #[test]
    fn between_and_null_checks() {
        let (sql, args) = w()
            .between("age", 10, 20)
            .and(w().is_null("name"))
            .and(w().is_not_null("id"))
            .to_sql()
            .unwrap();
        assert!(sql.contains("BETWEEN ? AND ?"));
        assert!(sql.contains("IS NULL"));
        assert!(sql.contains("IS NOT NULL"));
        assert_eq!(args.len(), 2);
    }

    #[test]
    fn like_variants() {
        let (sql, args) = w()
            .like("name", "%a%")
            .and(w().like_left("name", "a"))
            .and(w().like_right("name", "a"))
            .and(w().not_like("name", "%x%"))
            .to_sql()
            .unwrap();
        assert!(sql.contains("LIKE ?"));
        assert!(sql.contains("NOT LIKE ?"));
        assert_eq!(args.len(), 4);
        assert_eq!(args[1], SqlValue::String("%a".into()));
        assert_eq!(args[2], SqlValue::String("a%".into()));
    }

    #[test]
    fn not_eq_and_not_in() {
        let (sql, args) = w()
            .not_eq("status", 0)
            .and(w().not_in_list("id", [1i64, 2]))
            .to_sql()
            .unwrap();
        assert!(sql.contains("<> ?"));
        assert!(sql.contains("NOT IN"));
        assert_eq!(args.len(), 3);
    }

    #[test]
    fn native_and_not() {
        let (sql, args) = w()
            .eq("id", 1)
            .not()
            .native("custom = ?", [42i64])
            .to_sql()
            .unwrap();
        assert!(sql.starts_with("NOT ("));
        assert!(sql.contains("custom = ?"));
        assert_eq!(args.len(), 2);
    }

    #[test]
    fn model_and_primary_key_helpers() {
        let cond = TestUserWhere {
            name: Some("n".into()),
            age: Some(3),
        };
        let (sql, args) = w()
            .model(&cond)
            .primary_key::<crate::test_util::TestUser>(9i64)
            .filter_primary_key::<crate::test_util::TestUser>()
            .to_sql()
            .unwrap();
        assert!(sql.contains("name = ?"));
        assert!(sql.contains("age = ?"));
        assert!(sql.contains("id = ?"));
        assert!(sql.contains("id IS NOT NULL"));
        assert_eq!(args.len(), 3);
    }

    #[test]
    fn conditional_helpers_skip_when_disabled() {
        let (sql, args) = w()
            .eq_if("id", 1, false)
            .native_if(false, "x = ?", [1i64])
            .to_sql()
            .unwrap();
        assert!(sql.is_empty());
        assert!(args.is_empty());
    }

    #[test]
    fn empty_in_list_errors() {
        let err = w().in_list("id", Vec::<i64>::new()).to_sql().unwrap_err();
        assert!(matches!(err, LdbError::SqlBuild(_)));
    }
}

//! 单表自定义查询构建器。

use crate::engine::Engine;
use crate::error::LdbError;
use crate::model::LdbModel;
use crate::order::Order;
use crate::sql_build::BuiltSql;
use crate::where_builder::WhereBuilder;

/// 自定义 SELECT 构建器，支持单表、内连接、排序与分页。
pub struct QueryBuild<'a, E, T> {
    engine: &'a E,
    select_column_list: Vec<String>,
    table_name: Option<String>,
    join_list: Vec<(String, String)>,
    where_builder: Option<WhereBuilder>,
    order_by_list: Vec<(String, Order)>,
    limit: Option<u64>,
    offset: Option<u64>,
    _marker: std::marker::PhantomData<T>,
}

/// 创建模型查询构建器。
pub fn query_build<'a, T, E>(engine: &'a E) -> QueryBuild<'a, E, T>
where
    E: Engine,
    T: LdbModel,
{
    QueryBuild {
        engine,
        select_column_list: vec![],
        table_name: None,
        join_list: vec![],
        where_builder: None,
        order_by_list: vec![],
        limit: None,
        offset: None,
        _marker: std::marker::PhantomData,
    }
}

impl<'a, E, T> QueryBuild<'a, E, T>
where
    E: Engine,
    T: LdbModel + Default,
{
    pub fn select(mut self, column_list: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.select_column_list = column_list.into_iter().map(Into::into).collect();
        self
    }

    pub fn from(mut self, table_name: impl Into<String>) -> Self {
        self.table_name = Some(table_name.into());
        self
    }

    pub fn inner_join(mut self, table_name: impl Into<String>, on: impl Into<String>) -> Self {
        self.join_list.push((table_name.into(), on.into()));
        self
    }

    pub fn where_(mut self, where_builder: WhereBuilder) -> Self {
        self.where_builder = Some(where_builder);
        self
    }

    pub fn order_by(mut self, column: impl Into<String>, order: Order) -> Self {
        self.order_by_list.push((column.into(), order));
        self
    }

    pub fn limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: u64) -> Self {
        self.offset = Some(offset);
        self
    }

    fn build(&self, count: bool) -> Result<BuiltSql, LdbError> {
        let dialect = self.engine.dialect();
        let select_sql = if count {
            "COUNT(*)".to_string()
        } else if self.select_column_list.is_empty() {
            T::column_meta_list()
                .iter()
                .map(|meta| dialect.escape_identifier(meta.column_name))
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            self.select_column_list
                .iter()
                .map(|column| dialect.escape_identifier(column))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let table = self
            .table_name
            .as_deref()
            .unwrap_or(T::table_conf().table_name);
        let mut sql = format!(
            "SELECT {select_sql} FROM {}",
            dialect.escape_identifier(table)
        );
        for (join_table, on) in &self.join_list {
            sql.push_str(" INNER JOIN ");
            sql.push_str(&dialect.escape_identifier(join_table));
            sql.push_str(" ON ");
            sql.push_str(on);
        }
        let (where_sql, arg_list) = self
            .where_builder
            .as_ref()
            .unwrap_or(&WhereBuilder::default())
            .to_sql_with_dialect(dialect)?;
        if !where_sql.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&where_sql);
        }
        if !count && !self.order_by_list.is_empty() {
            sql.push_str(" ORDER BY ");
            sql.push_str(
                &self
                    .order_by_list
                    .iter()
                    .map(|(column, order)| {
                        let direction = match order {
                            Order::Asc => "ASC",
                            Order::Desc => "DESC",
                        };
                        format!("{} {direction}", dialect.escape_identifier(column))
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
            );
        }
        if !count {
            if let Some(limit) = self.limit {
                sql.push_str(&format!(" LIMIT {limit}"));
            }
            if let Some(offset) = self.offset {
                sql.push_str(&format!(" OFFSET {offset}"));
            }
        }
        Ok(BuiltSql { sql, arg_list })
    }

    pub async fn list(self) -> Result<Vec<T>, LdbError> {
        let built = self.build(false)?;
        self.engine.fetch_models(&built).await
    }

    pub async fn first(mut self) -> Result<Option<T>, LdbError> {
        self.limit = Some(1);
        Ok(self.list().await?.into_iter().next())
    }

    pub async fn count(self) -> Result<u64, LdbError> {
        let built = self.build(true)?;
        self.engine.query_scalar_u64(&built).await
    }

    pub async fn page(mut self, page: u64, page_size: u64) -> Result<Vec<T>, LdbError> {
        self.limit = Some(page_size);
        self.offset = Some(page.saturating_sub(1).saturating_mul(page_size));
        self.list().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exec::MockExecutor;
    use crate::test_util::TestUser;
    use crate::where_builder::w;

    #[tokio::test]
    async fn query_build_list_and_count() {
        let mock = MockExecutor::default();
        let row_list = query_build::<TestUser, _>(&mock)
            .where_(w().gt("id", 0))
            .order_by("id", Order::Desc)
            .limit(10)
            .list()
            .await
            .unwrap();
        assert!(row_list.is_empty());
        assert!(mock.last_sql().sql.contains("ORDER BY `id` DESC"));

        let count = query_build::<TestUser, _>(&mock)
            .from("t_user")
            .where_(w().gt("id", 0))
            .count()
            .await
            .unwrap();
        assert_eq!(count, 1);
    }
}

//! Update Builder 与入口。

use std::future::Future;
use std::pin::Pin;

use crate::crud::common::BuilderFlags;
use crate::engine::Engine;
use crate::error::LdbError;
use crate::model::LdbModel;
use crate::sql_build::build_update;
use crate::sql_value::{IntoSqlValue, SqlValue};
use crate::where_builder::WhereBuilder;

/// 按主键更新 Builder。
pub struct UpdateByPkBuilder<'a, E, M> {
    engine: &'a E,
    model: &'a M,
    flags: BuilderFlags,
    extra_set_list: Vec<(String, SqlValue)>,
}

/// 条件更新 Builder。
pub struct UpdateBuilder<'a, E, M> {
    engine: &'a E,
    patch: &'a M,
    flags: BuilderFlags,
    where_builder: Option<WhereBuilder>,
    allow_full_table: bool,
    extra_set_list: Vec<(String, SqlValue)>,
}

pub fn update_by_primary_key<'a, E, M>(engine: &'a E, model: &'a M) -> UpdateByPkBuilder<'a, E, M>
where
    E: Engine,
    M: LdbModel,
{
    UpdateByPkBuilder {
        engine,
        model,
        flags: BuilderFlags::default(),
        extra_set_list: vec![],
    }
}

pub fn update<'a, E, M>(engine: &'a E, patch: &'a M) -> UpdateBuilder<'a, E, M>
where
    E: Engine,
    M: LdbModel,
{
    UpdateBuilder {
        engine,
        patch,
        flags: BuilderFlags::default(),
        where_builder: None,
        allow_full_table: false,
        extra_set_list: vec![],
    }
}

macro_rules! update_builder_methods {
    ($t:ty) => {
        impl<'a, E, M> $t
        where
            E: Engine,
            M: LdbModel,
        {
            pub fn table_name(mut self, name: impl Into<String>) -> Self {
                self.flags = self.flags.table_name(name);
                self
            }

            pub fn show_sql(mut self, enabled: bool) -> Self {
                self.flags = self.flags.show_sql(enabled);
                self
            }

            pub fn dry_run(mut self, enabled: bool) -> Self {
                self.flags = self.flags.dry_run(enabled);
                self
            }

            pub fn set_null(mut self, column: &str) -> Self {
                self.extra_set_list
                    .push((column.to_string(), SqlValue::Null));
                self
            }

            pub fn set(mut self, column: &str, value: impl IntoSqlValue) -> Self {
                self.extra_set_list
                    .push((column.to_string(), value.into_sql_value()));
                self
            }
        }
    };
}

update_builder_methods!(UpdateByPkBuilder<'a, E, M>);
update_builder_methods!(UpdateBuilder<'a, E, M>);

impl<'a, E, M> UpdateBuilder<'a, E, M>
where
    E: Engine,
    M: LdbModel,
{
    pub fn where_(mut self, wb: WhereBuilder) -> Self {
        self.where_builder = Some(wb);
        self
    }

    pub fn allow_full_table(mut self, enabled: bool) -> Self {
        self.allow_full_table = enabled;
        self
    }
}

impl<'a, E, M> IntoFuture for UpdateByPkBuilder<'a, E, M>
where
    E: Engine,
    M: LdbModel,
{
    type Output = Result<u64, LdbError>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let table = self.flags.resolve_table_name::<M>();
            let pk = M::table_conf().primary_key_column_name_list;
            let mut wb = WhereBuilder::default();
            for col in pk {
                if let Some(v) = self.model.field_sql_value(
                    M::column_meta_list()
                        .iter()
                        .find(|m| m.column_name == *col)
                        .map(|m| m.field_name)
                        .unwrap_or(col),
                ) {
                    wb = wb.eq(col, v);
                }
            }
            let built = build_update(&table, self.model, &wb, &self.extra_set_list)?;
            run_update(self.engine, &self.flags, built).await
        })
    }
}

impl<'a, E, M> IntoFuture for UpdateBuilder<'a, E, M>
where
    E: Engine,
    M: LdbModel,
{
    type Output = Result<u64, LdbError>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let wb = self.where_builder.unwrap_or_default();
            if wb.is_empty() && !self.allow_full_table {
                return Err(LdbError::FullTableOpNotAllowed);
            }
            let table = self.flags.resolve_table_name::<M>();
            let built = build_update(&table, self.patch, &wb, &self.extra_set_list)?;
            run_update(self.engine, &self.flags, built).await
        })
    }
}

async fn run_update<E: Engine>(
    engine: &E,
    flags: &BuilderFlags,
    built: crate::sql_build::BuiltSql,
) -> Result<u64, LdbError> {
    if flags.show_sql {
        eprintln!("SQL: {} {:?}", built.sql, built.arg_list);
    }
    if flags.dry_run {
        return Ok(0);
    }
    engine.execute_built(&built).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exec::MockExecutor;
    use crate::test_util::TestUser;
    use crate::where_builder::w;

    #[tokio::test]
    async fn update_requires_where() {
        let mock = MockExecutor::default();
        let patch = TestUser::default();
        let err = update(&mock, &patch).await.unwrap_err();
        assert!(matches!(err, LdbError::FullTableOpNotAllowed));
    }

    #[tokio::test]
    async fn update_with_where() {
        let mock = MockExecutor::default();
        let patch = TestUser {
            name: Some("x".into()),
            ..Default::default()
        };
        let n = update(&mock, &patch).where_(w().eq("id", 1)).await.unwrap();
        assert_eq!(n, 1);
    }

    #[tokio::test]
    async fn update_by_primary_key_mock() {
        let mock = MockExecutor::default();
        let model = TestUser {
            id: Some(7),
            name: Some("pk".into()),
            age: None,
        };
        let n = update_by_primary_key(&mock, &model)
            .set_null("age")
            .await
            .unwrap();
        assert_eq!(n, 1);
        assert!(mock.last_sql().sql.contains("UPDATE"));
    }

    #[tokio::test]
    async fn update_allow_full_table_dry_run() {
        let mock = MockExecutor::default();
        let patch = TestUser {
            name: Some("x".into()),
            ..Default::default()
        };
        let n = update(&mock, &patch)
            .where_(w().gt("id", 0))
            .allow_full_table(true)
            .dry_run(true)
            .await
            .unwrap();
        assert_eq!(n, 0);
    }
}

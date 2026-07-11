//! Delete Builder 与入口。

use std::future::Future;
use std::pin::Pin;

use crate::crud::common::BuilderFlags;
use crate::engine::Engine;
use crate::error::LdbError;
use crate::model::LdbModel;
use crate::sql_build::build_delete;
use crate::where_builder::WhereBuilder;

/// 删除 Builder。
pub struct DeleteBuilder<'a, E, T> {
    engine: &'a E,
    _marker: std::marker::PhantomData<T>,
    flags: BuilderFlags,
    where_builder: Option<WhereBuilder>,
    allow_full_table: bool,
}

pub fn delete<'a, T, E>(engine: &'a E) -> DeleteBuilder<'a, E, T>
where
    T: LdbModel,
    E: Engine,
{
    DeleteBuilder {
        engine,
        _marker: std::marker::PhantomData,
        flags: BuilderFlags::default(),
        where_builder: None,
        allow_full_table: false,
    }
}

impl<'a, E, T> DeleteBuilder<'a, E, T>
where
    T: LdbModel,
    E: Engine,
{
    pub fn where_(mut self, wb: WhereBuilder) -> Self {
        self.where_builder = Some(wb);
        self
    }

    pub fn allow_full_table(mut self, enabled: bool) -> Self {
        self.allow_full_table = enabled;
        self
    }

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
}

impl<'a, E, T> IntoFuture for DeleteBuilder<'a, E, T>
where
    T: LdbModel,
    E: Engine,
{
    type Output = Result<u64, LdbError>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let wb = self.where_builder.unwrap_or_default();
            let table = self.flags.resolve_table_name::<T>();
            let built = build_delete(&table, &wb, self.allow_full_table)?;
            if self.flags.show_sql {
                eprintln!("SQL: {} {:?}", built.sql, built.arg_list);
            }
            if self.flags.dry_run {
                return Ok(0);
            }
            self.engine.execute_built(&built).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exec::MockExecutor;
    use crate::test_util::TestUser;
    use crate::where_builder::w;

    #[tokio::test]
    async fn delete_full_table_blocked() {
        let mock = MockExecutor::default();
        let err = delete::<TestUser, _>(&mock).where_(w()).await.unwrap_err();
        assert!(matches!(err, LdbError::FullTableOpNotAllowed));
    }

    #[tokio::test]
    async fn delete_by_id() {
        let mock = MockExecutor::default();
        let n = delete::<TestUser, _>(&mock)
            .where_(w().eq("id", 1))
            .await
            .unwrap();
        assert_eq!(n, 1);
    }

    #[tokio::test]
    async fn delete_dry_run_and_allow_full_table() {
        let mock = MockExecutor::default();
        let n = delete::<TestUser, _>(&mock)
            .allow_full_table(true)
            .dry_run(true)
            .await
            .unwrap();
        assert_eq!(n, 0);
    }
}

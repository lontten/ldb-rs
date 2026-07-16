//! Insert Builder 与入口。

use std::future::Future;
use std::pin::Pin;

use crate::crud::common::BuilderFlags;
use crate::engine::{Engine, InsertResult};
use crate::error::LdbError;
use crate::model::LdbModel;
use crate::on_conflict::OnConflict;
use crate::sql_build::build_insert;

/// `insert(engine, model)` 返回的 Builder。
pub struct InsertBuilder<'a, E, M> {
    engine: &'a E,
    model: &'a mut M,
    flags: BuilderFlags,
    on_conflict: Option<OnConflict>,
}

/// 插入一行。
pub fn insert<'a, E, M>(engine: &'a E, model: &'a mut M) -> InsertBuilder<'a, E, M>
where
    E: Engine,
    M: LdbModel,
{
    InsertBuilder {
        engine,
        model,
        flags: BuilderFlags::default(),
        on_conflict: None,
    }
}

impl<'a, E, M> InsertBuilder<'a, E, M>
where
    E: Engine,
    M: LdbModel,
{
    pub fn table_name(mut self, name: impl Into<String>) -> Self {
        self.flags = self.flags.table_name(name);
        self
    }

    pub fn on_conflict(mut self, action: OnConflict) -> Self {
        self.on_conflict = Some(action);
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

impl<'a, E, M> IntoFuture for InsertBuilder<'a, E, M>
where
    E: Engine,
    M: LdbModel,
{
    type Output = Result<InsertResult, LdbError>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let table = self.flags.resolve_table_name::<M>();
            let built = build_insert(
                &table,
                &*self.model,
                self.on_conflict.as_ref(),
                self.engine.dialect(),
            )?;
            if self.flags.show_sql {
                eprintln!("SQL: {} {:?}", built.sql, built.arg_list);
            }
            if self.flags.dry_run {
                return Ok(InsertResult { rows_affected: 0 });
            }
            let result = self.engine.execute_insert(&built).await?;
            if let (Some(column), Some(id)) = (M::table_conf().auto_column, result.generated_id) {
                let field_name = M::column_meta_list()
                    .iter()
                    .find(|meta| meta.column_name == column)
                    .map(|meta| meta.field_name)
                    .ok_or_else(|| {
                        LdbError::ModelMapping(format!("自增列 `{column}` 没有对应字段"))
                    })?;
                let value = i64::try_from(id)
                    .map(crate::sql_value::SqlValue::I64)
                    .unwrap_or(crate::sql_value::SqlValue::U64(id));
                self.model.set_field_sql_value(field_name, value)?;
            }
            Ok(InsertResult {
                rows_affected: result.rows_affected,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exec::MockExecutor;
    use crate::test_util::TestUser;

    #[tokio::test]
    async fn insert_dry_run() {
        let mock = MockExecutor::default();
        let mut user = TestUser {
            id: None,
            name: Some("tom".into()),
            age: None,
        };
        let result = insert(&mock, &mut user).dry_run(true).await.unwrap();
        assert_eq!(result.rows_affected, 0);
    }

    #[tokio::test]
    async fn insert_mock_execute() {
        let mock = MockExecutor::default();
        let mut user = TestUser {
            id: None,
            name: Some("tom".into()),
            age: None,
        };
        mock.set_mock_generated_id(42);
        let result = insert(&mock, &mut user).await.unwrap();
        assert_eq!(result.rows_affected, 1);
        assert_eq!(user.id, Some(42));
        assert!(mock.last_sql().sql.contains("INSERT"));
    }

    #[tokio::test]
    async fn insert_on_conflict_dry_run() {
        let mock = MockExecutor::default();
        let mut user = TestUser {
            name: Some("u".into()),
            ..Default::default()
        };
        let result = insert(&mock, &mut user)
            .on_conflict(crate::on_conflict::OnConflict::DoNothing)
            .dry_run(true)
            .await
            .unwrap();
        assert_eq!(result.rows_affected, 0);
    }
}

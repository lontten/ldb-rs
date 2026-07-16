//! 原生 SQL 查询、执行与预编译语句封装。

use std::future::Future;
use std::pin::Pin;

use crate::engine::Engine;
use crate::error::LdbError;
use crate::model::LdbModel;
use crate::sql_build::BuiltSql;
use crate::sql_value::{IntoSqlValue, SqlValue};

fn built(sql: impl Into<String>, arg_list: Vec<SqlValue>) -> BuiltSql {
    BuiltSql {
        sql: sql.into(),
        arg_list,
    }
}

/// 执行原生 SQL 并映射为模型列表。
pub async fn native_query<T, E>(
    engine: &E,
    sql: impl Into<String>,
    arg_list: impl IntoIterator<Item = impl IntoSqlValue>,
) -> Result<Vec<T>, LdbError>
where
    T: LdbModel + Default,
    E: Engine,
{
    let built = built(
        sql,
        arg_list
            .into_iter()
            .map(IntoSqlValue::into_sql_value)
            .collect(),
    );
    engine.fetch_models(&built).await
}

/// 执行原生 DML 并返回受影响行数。
pub async fn native_exec<E>(
    engine: &E,
    sql: impl Into<String>,
    arg_list: impl IntoIterator<Item = impl IntoSqlValue>,
) -> Result<u64, LdbError>
where
    E: Engine,
{
    let built = built(
        sql,
        arg_list
            .into_iter()
            .map(IntoSqlValue::into_sql_value)
            .collect(),
    );
    engine.execute_built(&built).await
}

/// 已准备的 SQL 模板；底层连接池复用 sqlx 的语句缓存。
pub struct PreparedStatement<'a, E> {
    engine: &'a E,
    sql: String,
}

/// 准备可重复执行的 SQL 模板。
pub async fn prepare<'a, E>(
    engine: &'a E,
    sql: impl Into<String>,
) -> Result<PreparedStatement<'a, E>, LdbError>
where
    E: Engine,
{
    Ok(PreparedStatement {
        engine,
        sql: sql.into(),
    })
}

/// 已绑定参数的预编译查询。
pub struct StmtQuery<'a, E, T> {
    statement: &'a PreparedStatement<'a, E>,
    arg_list: Vec<SqlValue>,
    _marker: std::marker::PhantomData<T>,
}

impl<'a, E> PreparedStatement<'a, E>
where
    E: Engine,
{
    pub fn query<T>(
        &'a self,
        arg_list: impl IntoIterator<Item = impl IntoSqlValue>,
    ) -> StmtQuery<'a, E, T>
    where
        T: LdbModel + Default,
    {
        StmtQuery {
            statement: self,
            arg_list: arg_list
                .into_iter()
                .map(IntoSqlValue::into_sql_value)
                .collect(),
            _marker: std::marker::PhantomData,
        }
    }

    pub async fn execute(
        &self,
        arg_list: impl IntoIterator<Item = impl IntoSqlValue>,
    ) -> Result<u64, LdbError> {
        native_exec(self.engine, self.sql.clone(), arg_list).await
    }
}

impl<'a, E, T> IntoFuture for StmtQuery<'a, E, T>
where
    E: Engine,
    T: LdbModel + Default,
{
    type Output = Result<Vec<T>, LdbError>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let built = built(self.statement.sql.clone(), self.arg_list);
            self.statement.engine.fetch_models(&built).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exec::MockExecutor;
    use crate::test_util::TestUser;

    #[tokio::test]
    async fn native_exec_and_prepared_query() {
        let mock = MockExecutor::default();
        assert_eq!(
            native_exec(&mock, "UPDATE t SET name = ? WHERE id = ?", ["n", "1"])
                .await
                .unwrap(),
            1
        );
        mock.set_mock_rows(vec![vec![
            ("id".into(), SqlValue::I64(1)),
            ("name".into(), SqlValue::String("n".into())),
            ("age".into(), SqlValue::I64(2)),
        ]]);
        let statement = prepare(&mock, "SELECT id, name, age FROM t WHERE id = ?")
            .await
            .unwrap();
        let row_list = statement.query::<TestUser>([1i64]).await.unwrap();
        assert_eq!(row_list.len(), 1);
    }
}

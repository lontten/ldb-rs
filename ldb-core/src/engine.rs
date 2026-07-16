//! 数据库执行引擎与事务。

use tokio::sync::Mutex;

use crate::error::LdbError;
use crate::exec::{DbKind, InsertExecution, MockExecutor, MysqlEngine, PgEngine, SqlExecutor};

/// 插入结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InsertResult {
    pub rows_affected: u64,
}

/// 可执行 SQL 的数据库引擎（连接池或事务）。
pub trait Engine: SqlExecutor + Send + Sync {
    fn ping(&self) -> impl Future<Output = Result<(), LdbError>> + Send;
    fn begin(&self) -> impl Future<Output = Result<Transaction, LdbError>> + Send;
}

impl Engine for MysqlEngine {
    async fn ping(&self) -> Result<(), LdbError> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(())
    }

    async fn begin(&self) -> Result<Transaction, LdbError> {
        let tx = self.pool.begin().await?;
        Ok(Transaction::wrap_mysql(tx, *self.dialect))
    }
}

impl Engine for PgEngine {
    async fn ping(&self) -> Result<(), LdbError> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(())
    }

    async fn begin(&self) -> Result<Transaction, LdbError> {
        let tx = self.pool.begin().await?;
        Ok(Transaction::wrap_pg(tx))
    }
}

/// 事务句柄；实现 `Engine` 与 `SqlExecutor`。
pub enum Transaction {
    Mysql(
        Mutex<sqlx::Transaction<'static, sqlx::MySql>>,
        crate::dialect::mysql_dialect::MysqlDialect,
    ),
    Pg(Mutex<sqlx::Transaction<'static, sqlx::Postgres>>),
}

impl Transaction {
    fn wrap_mysql(
        tx: sqlx::Transaction<'static, sqlx::MySql>,
        dialect: crate::dialect::mysql_dialect::MysqlDialect,
    ) -> Self {
        Transaction::Mysql(Mutex::new(tx), dialect)
    }

    fn wrap_pg(tx: sqlx::Transaction<'static, sqlx::Postgres>) -> Self {
        Transaction::Pg(Mutex::new(tx))
    }

    pub async fn commit(self) -> Result<(), LdbError> {
        match self {
            Transaction::Mysql(tx, _) => tx.into_inner().commit().await?,
            Transaction::Pg(tx) => tx.into_inner().commit().await?,
        }
        Ok(())
    }

    pub async fn rollback(self) -> Result<(), LdbError> {
        match self {
            Transaction::Mysql(tx, _) => tx.into_inner().rollback().await?,
            Transaction::Pg(tx) => tx.into_inner().rollback().await?,
        }
        Ok(())
    }
}

impl SqlExecutor for Transaction {
    fn db_kind(&self) -> DbKind {
        match self {
            Transaction::Mysql(_, _) => DbKind::Mysql,
            Transaction::Pg(_) => DbKind::Pg,
        }
    }

    fn dialect(&self) -> &dyn crate::dialect::dialect::Dialect {
        match self {
            Transaction::Mysql(_, dialect) => dialect,
            Transaction::Pg(_) => {
                static D: crate::dialect::pg_dialect::PgDialect =
                    crate::dialect::pg_dialect::PgDialect;
                &D
            }
        }
    }

    async fn execute_built(&self, built: &crate::sql_build::BuiltSql) -> Result<u64, LdbError> {
        crate::exec::validate_built_args(built, self.db_kind())?;
        match self {
            Transaction::Mysql(tx, _) => {
                let sql = crate::sql_build::dialect_exec_sql(self.dialect(), built);
                let mut q = sqlx::query(&sql);
                for v in &built.arg_list {
                    q = crate::exec::bind_mysql(q, v);
                }
                let mut guard = tx.lock().await;
                Ok(q.execute(&mut **guard).await?.rows_affected())
            }
            Transaction::Pg(tx) => {
                let sql = crate::sql_build::dialect_exec_sql(self.dialect(), built);
                let mut q = sqlx::query(&sql);
                for v in &built.arg_list {
                    q = crate::exec::bind_pg(q, v);
                }
                let mut guard = tx.lock().await;
                Ok(q.execute(&mut **guard).await?.rows_affected())
            }
        }
    }

    async fn execute_insert(
        &self,
        built: &crate::sql_build::BuiltSql,
    ) -> Result<InsertExecution, LdbError> {
        crate::exec::validate_built_args(built, self.db_kind())?;
        match self {
            Transaction::Mysql(tx, _) => {
                let sql = crate::sql_build::dialect_exec_sql(self.dialect(), built);
                let mut q = sqlx::query(&sql);
                for value in &built.arg_list {
                    q = crate::exec::bind_mysql(q, value);
                }
                let mut guard = tx.lock().await;
                let result = q.execute(&mut **guard).await?;
                let generated_id = result.last_insert_id();
                Ok(InsertExecution {
                    rows_affected: result.rows_affected(),
                    generated_id: (generated_id > 0).then_some(generated_id),
                })
            }
            Transaction::Pg(tx) => {
                let sql = crate::sql_build::dialect_exec_sql(self.dialect(), built);
                let mut q = sqlx::query(&sql);
                for value in &built.arg_list {
                    q = crate::exec::bind_pg(q, value);
                }
                let mut guard = tx.lock().await;
                if built.sql.contains(" RETURNING ") {
                    use sqlx::Row;
                    let row = q.fetch_optional(&mut **guard).await?;
                    match row {
                        Some(row) => {
                            let id: i64 = row.try_get(0)?;
                            Ok(InsertExecution {
                                rows_affected: 1,
                                generated_id: Some(id as u64),
                            })
                        }
                        None => Ok(InsertExecution {
                            rows_affected: 0,
                            generated_id: None,
                        }),
                    }
                } else {
                    let result = q.execute(&mut **guard).await?;
                    Ok(InsertExecution {
                        rows_affected: result.rows_affected(),
                        generated_id: None,
                    })
                }
            }
        }
    }

    async fn query_rows(&self, built: &crate::sql_build::BuiltSql) -> Result<u64, LdbError> {
        crate::exec::validate_built_args(built, self.db_kind())?;
        match self {
            Transaction::Mysql(tx, _) => {
                let sql = crate::sql_build::dialect_exec_sql(self.dialect(), built);
                let mut q = sqlx::query(&sql);
                for v in &built.arg_list {
                    q = crate::exec::bind_mysql(q, v);
                }
                let mut guard = tx.lock().await;
                Ok(q.fetch_all(&mut **guard).await?.len() as u64)
            }
            Transaction::Pg(tx) => {
                let sql = crate::sql_build::dialect_exec_sql(self.dialect(), built);
                let mut q = sqlx::query(&sql);
                for v in &built.arg_list {
                    q = crate::exec::bind_pg(q, v);
                }
                let mut guard = tx.lock().await;
                Ok(q.fetch_all(&mut **guard).await?.len() as u64)
            }
        }
    }

    async fn fetch_models<T: crate::model::LdbModel + Default>(
        &self,
        built: &crate::sql_build::BuiltSql,
    ) -> Result<Vec<T>, LdbError> {
        crate::exec::validate_built_args(built, self.db_kind())?;
        match self {
            Transaction::Mysql(tx, _) => {
                let sql = crate::sql_build::dialect_exec_sql(self.dialect(), built);
                let mut q = sqlx::query(&sql);
                for v in &built.arg_list {
                    q = crate::exec::bind_mysql(q, v);
                }
                let mut guard = tx.lock().await;
                let rows = q.fetch_all(&mut **guard).await?;
                let column_name_list: Vec<String> = T::column_meta_list()
                    .iter()
                    .map(|m| m.column_name.to_string())
                    .collect();
                let mut out = Vec::with_capacity(rows.len());
                for row in rows {
                    let column_value_list: Vec<(String, crate::sql_value::SqlValue)> =
                        column_name_list
                            .iter()
                            .map(|col| {
                                crate::exec::mysql_read_column(&row, col)
                                    .map(|val| (col.clone(), val))
                            })
                            .collect::<Result<_, _>>()?;
                    out.push(crate::model::hydrate_model(&column_value_list)?);
                }
                Ok(out)
            }
            Transaction::Pg(tx) => {
                let mut guard = tx.lock().await;
                let sql = crate::sql_build::dialect_exec_sql(self.dialect(), built);
                let mut q = sqlx::query(&sql);
                for v in &built.arg_list {
                    q = crate::exec::bind_pg(q, v);
                }
                let rows = q.fetch_all(&mut **guard).await?;
                let column_name_list: Vec<String> = T::column_meta_list()
                    .iter()
                    .map(|m| m.column_name.to_string())
                    .collect();
                let mut out = Vec::with_capacity(rows.len());
                for row in rows {
                    let column_value_list: Vec<(String, crate::sql_value::SqlValue)> =
                        column_name_list
                            .iter()
                            .map(|col| {
                                crate::exec::pg_read_column(&row, col).map(|val| (col.clone(), val))
                            })
                            .collect::<Result<_, _>>()?;
                    out.push(crate::model::hydrate_model(&column_value_list)?);
                }
                Ok(out)
            }
        }
    }

    async fn query_scalar_u64(&self, built: &crate::sql_build::BuiltSql) -> Result<u64, LdbError> {
        crate::exec::validate_built_args(built, self.db_kind())?;
        match self {
            Transaction::Mysql(tx, _) => {
                let sql = crate::sql_build::dialect_exec_sql(self.dialect(), built);
                let mut q = sqlx::query(&sql);
                for v in &built.arg_list {
                    q = crate::exec::bind_mysql(q, v);
                }
                let mut guard = tx.lock().await;
                let row = q.fetch_one(&mut **guard).await?;
                crate::exec::mysql_row_first_u64(&row)
            }
            Transaction::Pg(tx) => {
                let sql = crate::sql_build::dialect_exec_sql(self.dialect(), built);
                let mut q = sqlx::query(&sql);
                for v in &built.arg_list {
                    q = crate::exec::bind_pg(q, v);
                }
                let mut guard = tx.lock().await;
                let row = q.fetch_one(&mut **guard).await?;
                crate::exec::pg_row_first_u64(&row)
            }
        }
    }

    async fn query_exists(&self, built: &crate::sql_build::BuiltSql) -> Result<bool, LdbError> {
        crate::exec::validate_built_args(built, self.db_kind())?;
        match self {
            Transaction::Mysql(tx, _) => {
                let sql = crate::sql_build::dialect_exec_sql(self.dialect(), built);
                let mut q = sqlx::query(&sql);
                for v in &built.arg_list {
                    q = crate::exec::bind_mysql(q, v);
                }
                let mut guard = tx.lock().await;
                Ok(q.fetch_optional(&mut **guard).await?.is_some())
            }
            Transaction::Pg(tx) => {
                let sql = crate::sql_build::dialect_exec_sql(self.dialect(), built);
                let mut q = sqlx::query(&sql);
                for v in &built.arg_list {
                    q = crate::exec::bind_pg(q, v);
                }
                let mut guard = tx.lock().await;
                Ok(q.fetch_optional(&mut **guard).await?.is_some())
            }
        }
    }
}

impl Engine for Transaction {
    async fn ping(&self) -> Result<(), LdbError> {
        Ok(())
    }

    async fn begin(&self) -> Result<Transaction, LdbError> {
        Err(LdbError::SqlBuild("事务内不能嵌套 begin".into()))
    }
}

impl Engine for MockExecutor {
    async fn ping(&self) -> Result<(), LdbError> {
        Ok(())
    }

    async fn begin(&self) -> Result<Transaction, LdbError> {
        Err(LdbError::NotImplemented)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_begin_not_implemented() {
        let mock = MockExecutor::default();
        assert!(matches!(mock.begin().await, Err(LdbError::NotImplemented)));
    }
}

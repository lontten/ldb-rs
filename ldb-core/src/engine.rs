//! 数据库执行引擎与事务。

use tokio::sync::Mutex;

use crate::error::LdbError;
use crate::exec::{DbKind, MockExecutor, MysqlEngine, PgEngine, SqlExecutor};

/// 插入结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InsertResult {
    pub rows_affected: u64,
}

/// 可执行 SQL 的数据库引擎（连接池或事务）。
pub trait Engine: SqlExecutor + Send + Sync {
    fn ping(&self) -> impl std::future::Future<Output = Result<(), LdbError>> + Send;
    fn begin(&self) -> impl std::future::Future<Output = Result<Transaction, LdbError>> + Send;
}

impl Engine for MysqlEngine {
    async fn ping(&self) -> Result<(), LdbError> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(())
    }

    async fn begin(&self) -> Result<Transaction, LdbError> {
        let tx = self.pool.begin().await?;
        Ok(Transaction::wrap_mysql(tx))
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
    Mysql(Mutex<sqlx::Transaction<'static, sqlx::MySql>>),
    Pg(Mutex<sqlx::Transaction<'static, sqlx::Postgres>>),
}

impl Transaction {
    fn wrap_mysql(tx: sqlx::Transaction<'_, sqlx::MySql>) -> Self {
        Transaction::Mysql(Mutex::new(unsafe {
            std::mem::transmute::<
                sqlx::Transaction<'_, sqlx::MySql>,
                sqlx::Transaction<'static, sqlx::MySql>,
            >(tx)
        }))
    }

    fn wrap_pg(tx: sqlx::Transaction<'_, sqlx::Postgres>) -> Self {
        Transaction::Pg(Mutex::new(unsafe {
            std::mem::transmute::<
                sqlx::Transaction<'_, sqlx::Postgres>,
                sqlx::Transaction<'static, sqlx::Postgres>,
            >(tx)
        }))
    }

    pub async fn commit(self) -> Result<(), LdbError> {
        match self {
            Transaction::Mysql(tx) => tx.into_inner().commit().await?,
            Transaction::Pg(tx) => tx.into_inner().commit().await?,
        }
        Ok(())
    }

    pub async fn rollback(self) -> Result<(), LdbError> {
        match self {
            Transaction::Mysql(tx) => tx.into_inner().rollback().await?,
            Transaction::Pg(tx) => tx.into_inner().rollback().await?,
        }
        Ok(())
    }
}

impl SqlExecutor for Transaction {
    fn db_kind(&self) -> DbKind {
        match self {
            Transaction::Mysql(_) => DbKind::Mysql,
            Transaction::Pg(_) => DbKind::Pg,
        }
    }

    fn dialect(&self) -> &dyn crate::dialect::Dialect {
        match self {
            Transaction::Mysql(_) => {
                static D: crate::mysql_dialect::MysqlDialect = crate::mysql_dialect::MysqlDialect;
                &D
            }
            Transaction::Pg(_) => {
                static D: crate::pg_dialect::PgDialect = crate::pg_dialect::PgDialect;
                &D
            }
        }
    }

    async fn execute_built(&self, built: &crate::sql_build::BuiltSql) -> Result<u64, LdbError> {
        match self {
            Transaction::Mysql(tx) => {
                let (sql, _) = crate::sql_build::dialect_exec_sql(self.dialect(), built, false);
                let mut q = sqlx::query(&sql);
                for v in &built.arg_list {
                    q = crate::exec::bind_mysql(q, v);
                }
                let mut guard = tx.lock().await;
                Ok(q.execute(&mut **guard).await?.rows_affected())
            }
            Transaction::Pg(tx) => {
                let (sql, _) = crate::sql_build::dialect_exec_sql(self.dialect(), built, false);
                let mut q = sqlx::query(&sql);
                for v in &built.arg_list {
                    q = crate::exec::bind_pg(q, v);
                }
                let mut guard = tx.lock().await;
                Ok(q.execute(&mut **guard).await?.rows_affected())
            }
        }
    }

    async fn query_rows(&self, built: &crate::sql_build::BuiltSql) -> Result<u64, LdbError> {
        match self {
            Transaction::Mysql(tx) => {
                let (sql, _) = crate::sql_build::dialect_exec_sql(self.dialect(), built, true);
                let mut q = sqlx::query(&sql);
                for v in &built.arg_list {
                    q = crate::exec::bind_mysql(q, v);
                }
                let mut guard = tx.lock().await;
                Ok(q.fetch_all(&mut **guard).await?.len() as u64)
            }
            Transaction::Pg(tx) => {
                let (sql, _) = crate::sql_build::dialect_exec_sql(self.dialect(), built, true);
                let mut q = sqlx::query(&sql);
                for v in &built.arg_list {
                    q = crate::exec::bind_pg(q, v);
                }
                let mut guard = tx.lock().await;
                Ok(q.fetch_all(&mut **guard).await?.len() as u64)
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

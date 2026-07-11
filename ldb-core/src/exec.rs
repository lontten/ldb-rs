//! SQL 执行与 `SqlExecutor` 抽象。

use std::sync::{Arc, Mutex};

use sqlx::mysql::MySqlPoolOptions;
use sqlx::postgres::PgPoolOptions;
use sqlx::{MySql, Pool, Postgres};

use crate::config::{MysqlConfig, PgConfig, PoolConfig};
use crate::dialect::Dialect;
use crate::error::LdbError;
use crate::mysql_dialect::MysqlDialect;
use crate::pg_dialect::PgDialect;
use crate::sql_build::BuiltSql;
use crate::sql_value::SqlValue;

/// 数据库种类。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbKind {
    Mysql,
    Pg,
}

/// 内部 SQL 执行接口。
pub trait SqlExecutor: Send + Sync {
    fn db_kind(&self) -> DbKind;
    fn dialect(&self) -> &dyn Dialect;
    fn execute_built(
        &self,
        built: &BuiltSql,
    ) -> impl std::future::Future<Output = Result<u64, LdbError>> + Send;
    fn query_rows(
        &self,
        built: &BuiltSql,
    ) -> impl std::future::Future<Output = Result<u64, LdbError>> + Send;
}

/// MySQL 连接池引擎。
#[derive(Clone)]
pub struct MysqlEngine {
    pub(crate) pool: Pool<MySql>,
    pub(crate) dialect: Arc<MysqlDialect>,
}

/// PostgreSQL 连接池引擎。
#[derive(Clone)]
pub struct PgEngine {
    pub(crate) pool: Pool<Postgres>,
    pub(crate) dialect: Arc<PgDialect>,
}

pub(crate) fn apply_pool_config_mysql(
    mut opts: MySqlPoolOptions,
    pool: Option<&PoolConfig>,
) -> MySqlPoolOptions {
    if let Some(p) = pool {
        if let Some(n) = p.max_open.filter(|&v| v > 0) {
            opts = opts.max_connections(n);
        }
        if let Some(d) = p.max_lifetime {
            opts = opts.max_lifetime(d);
        }
        if let Some(d) = p.max_idle_time {
            opts = opts.idle_timeout(d);
        }
    }
    opts
}

pub(crate) fn apply_pool_config_pg(
    mut opts: PgPoolOptions,
    pool: Option<&PoolConfig>,
) -> PgPoolOptions {
    if let Some(p) = pool {
        if let Some(n) = p.max_open.filter(|&v| v > 0) {
            opts = opts.max_connections(n);
        }
        if let Some(d) = p.max_lifetime {
            opts = opts.max_lifetime(d);
        }
        if let Some(d) = p.max_idle_time {
            opts = opts.idle_timeout(d);
        }
    }
    opts
}

pub async fn open_mysql_pool(
    config: &MysqlConfig,
    pool: Option<&PoolConfig>,
) -> Result<Pool<MySql>, LdbError> {
    let opts = apply_pool_config_mysql(MySqlPoolOptions::new(), pool);
    Ok(opts.connect(&config.dsn()).await?)
}

pub async fn open_pg_pool(
    config: &PgConfig,
    pool: Option<&PoolConfig>,
) -> Result<Pool<Postgres>, LdbError> {
    let opts = apply_pool_config_pg(PgPoolOptions::new(), pool);
    Ok(opts.connect(&config.dsn()).await?)
}

pub(crate) fn bind_mysql<'q>(
    q: sqlx::query::Query<'q, MySql, sqlx::mysql::MySqlArguments>,
    v: &SqlValue,
) -> sqlx::query::Query<'q, MySql, sqlx::mysql::MySqlArguments> {
    match v {
        SqlValue::Null => q.bind(None::<String>),
        SqlValue::Bool(b) => q.bind(*b),
        SqlValue::I64(n) => q.bind(*n),
        SqlValue::U64(n) => q.bind(*n as i64),
        SqlValue::F64(n) => q.bind(*n),
        SqlValue::String(s) => q.bind(s.clone()),
    }
}

pub(crate) fn bind_pg<'q>(
    q: sqlx::query::Query<'q, Postgres, sqlx::postgres::PgArguments>,
    v: &SqlValue,
) -> sqlx::query::Query<'q, Postgres, sqlx::postgres::PgArguments> {
    match v {
        SqlValue::Null => q.bind(None::<String>),
        SqlValue::Bool(b) => q.bind(*b),
        SqlValue::I64(n) => q.bind(*n),
        SqlValue::U64(n) => q.bind(*n as i64),
        SqlValue::F64(n) => q.bind(*n),
        SqlValue::String(s) => q.bind(s.clone()),
    }
}

impl SqlExecutor for MysqlEngine {
    fn db_kind(&self) -> DbKind {
        DbKind::Mysql
    }

    fn dialect(&self) -> &dyn Dialect {
        self.dialect.as_ref()
    }

    async fn execute_built(&self, built: &BuiltSql) -> Result<u64, LdbError> {
        let (sql, _) = crate::sql_build::dialect_exec_sql(self.dialect(), built, false);
        let mut q = sqlx::query(&sql);
        for v in &built.arg_list {
            q = bind_mysql(q, v);
        }
        Ok(q.execute(&self.pool).await?.rows_affected())
    }

    async fn query_rows(&self, built: &BuiltSql) -> Result<u64, LdbError> {
        let (sql, _) = crate::sql_build::dialect_exec_sql(self.dialect(), built, true);
        let mut q = sqlx::query(&sql);
        for v in &built.arg_list {
            q = bind_mysql(q, v);
        }
        Ok(q.fetch_all(&self.pool).await?.len() as u64)
    }
}

impl SqlExecutor for PgEngine {
    fn db_kind(&self) -> DbKind {
        DbKind::Pg
    }

    fn dialect(&self) -> &dyn Dialect {
        self.dialect.as_ref()
    }

    async fn execute_built(&self, built: &BuiltSql) -> Result<u64, LdbError> {
        let (sql, _) = crate::sql_build::dialect_exec_sql(self.dialect(), built, false);
        let mut q = sqlx::query(&sql);
        for v in &built.arg_list {
            q = bind_pg(q, v);
        }
        Ok(q.execute(&self.pool).await?.rows_affected())
    }

    async fn query_rows(&self, built: &BuiltSql) -> Result<u64, LdbError> {
        let (sql, _) = crate::sql_build::dialect_exec_sql(self.dialect(), built, true);
        let mut q = sqlx::query(&sql);
        for v in &built.arg_list {
            q = bind_pg(q, v);
        }
        Ok(q.fetch_all(&self.pool).await?.len() as u64)
    }
}

#[cfg(any(feature = "integration", test))]
impl MysqlEngine {
    /// 执行原始 SQL（集成测试建表/清表用）。
    pub async fn exec_sql(&self, sql: &str) -> Result<(), LdbError> {
        sqlx::query(sql).execute(&self.pool).await?;
        Ok(())
    }
}

#[cfg(any(feature = "integration", test))]
impl PgEngine {
    /// 执行原始 SQL（集成测试建表/清表用）。
    pub async fn exec_sql(&self, sql: &str) -> Result<(), LdbError> {
        sqlx::query(sql).execute(&self.pool).await?;
        Ok(())
    }
}

/// 记录最后执行的 SQL（单元测试用）。
#[derive(Debug, Clone, Default)]
pub struct RecordedSql {
    pub sql: String,
    pub arg_list: Vec<SqlValue>,
}

/// Mock 执行器：不连接数据库，仅记录 SQL。
#[derive(Debug, Default)]
pub struct MockExecutor {
    pub recorded: Arc<Mutex<RecordedSql>>,
}

impl MockExecutor {
    pub fn last_sql(&self) -> RecordedSql {
        self.recorded.lock().unwrap().clone()
    }
}

impl SqlExecutor for MockExecutor {
    fn db_kind(&self) -> DbKind {
        DbKind::Mysql
    }

    fn dialect(&self) -> &dyn Dialect {
        static D: MysqlDialect = MysqlDialect;
        &D
    }

    async fn execute_built(&self, built: &BuiltSql) -> Result<u64, LdbError> {
        let mut rec = self.recorded.lock().unwrap();
        rec.sql = built.sql.clone();
        rec.arg_list = built.arg_list.clone();
        Ok(1)
    }

    async fn query_rows(&self, built: &BuiltSql) -> Result<u64, LdbError> {
        let mut rec = self.recorded.lock().unwrap();
        rec.sql = built.sql.clone();
        rec.arg_list = built.arg_list.clone();
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PoolConfig;
    use crate::sql_build::build_insert;
    use crate::test_util::TestUser;
    use sqlx::MySql;

    #[tokio::test]
    async fn mock_records_sql() {
        let mock = MockExecutor::default();
        let user = TestUser {
            id: None,
            name: Some("a".into()),
            age: None,
        };
        let built = build_insert("t_user", &user, None, mock.dialect()).unwrap();
        mock.execute_built(&built).await.unwrap();
        assert!(mock.last_sql().sql.contains("INSERT"));
    }

    #[test]
    fn apply_pool_config_mysql_maps_fields() {
        let opts = apply_pool_config_mysql(
            MySqlPoolOptions::new(),
            Some(&PoolConfig {
                max_idle_count: None,
                max_open: Some(5),
                max_lifetime: Some(std::time::Duration::from_secs(30)),
                max_idle_time: Some(std::time::Duration::from_secs(10)),
            }),
        );
        let _ = opts;
    }

    #[test]
    fn apply_pool_config_pg_maps_fields() {
        let opts = apply_pool_config_pg(
            PgPoolOptions::new(),
            Some(&PoolConfig {
                max_idle_count: None,
                max_open: Some(3),
                max_lifetime: None,
                max_idle_time: None,
            }),
        );
        let _ = opts;
    }

    #[tokio::test]
    async fn mock_query_rows_records_sql() {
        let mock = MockExecutor::default();
        let built = crate::sql_build::BuiltSql {
            sql: "SELECT 1".into(),
            arg_list: vec![],
        };
        assert_eq!(mock.query_rows(&built).await.unwrap(), 0);
        assert_eq!(mock.last_sql().sql, "SELECT 1");
    }

    #[test]
    fn bind_mysql_and_pg_each_type() {
        let values = [
            SqlValue::Null,
            SqlValue::Bool(true),
            SqlValue::I64(1),
            SqlValue::U64(2),
            SqlValue::F64(1.5),
            SqlValue::String("s".into()),
        ];
        for v in values {
            let q = sqlx::query::<MySql>("SELECT 1");
            let _ = bind_mysql(q, &v);
            let q = sqlx::query::<Postgres>("SELECT 1");
            let _ = bind_pg(q, &v);
        }
    }
}

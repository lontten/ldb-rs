//! SQL 执行与 `SqlExecutor` 抽象。

use std::sync::{Arc, Mutex};

use sqlx::mysql::MySqlPoolOptions;
use sqlx::postgres::PgPoolOptions;
use sqlx::{MySql, Pool, Postgres};

use crate::config::{MysqlConfig, PgConfig, PoolConfig};
use crate::dialect::dialect::Dialect;
use crate::dialect::mysql_dialect::MysqlDialect;
use crate::dialect::pg_dialect::PgDialect;
use crate::error::LdbError;
use crate::model::{LdbModel, hydrate_model};
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

    /// 执行 DML（INSERT/UPDATE/DELETE），返回 `rows_affected`；不适用于 SELECT 标量或存在性查询。
    fn execute_built(
        &self,
        built: &BuiltSql,
    ) -> impl std::future::Future<Output = Result<u64, LdbError>> + Send;

    /// 执行 SELECT 多行查询，返回结果行数；供基准测试等使用。
    fn query_rows(
        &self,
        built: &BuiltSql,
    ) -> impl std::future::Future<Output = Result<u64, LdbError>> + Send;

    /// 执行 SELECT 并将结果行映射为模型列表；供 `first` / `list` 使用。
    fn fetch_models<T: LdbModel + Default>(
        &self,
        built: &BuiltSql,
    ) -> impl std::future::Future<Output = Result<Vec<T>, LdbError>> + Send;

    /// 执行 SELECT 标量查询（如 `COUNT(*)`），读取第一列数值；供 `count` 使用。
    fn query_scalar_u64(
        &self,
        built: &BuiltSql,
    ) -> impl std::future::Future<Output = Result<u64, LdbError>> + Send;

    /// 执行 SELECT 存在性查询（如 `SELECT 1 … LIMIT 1`），判断是否有匹配行；供 `has` 使用。
    fn query_exists(
        &self,
        built: &BuiltSql,
    ) -> impl std::future::Future<Output = Result<bool, LdbError>> + Send;
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

// 从 MySQL 查询行第一列解析 COUNT 标量（`i64` → `u64`）。
pub(crate) fn mysql_row_first_u64(row: &sqlx::mysql::MySqlRow) -> Result<u64, LdbError> {
    use sqlx::Row;
    let n: i64 = row
        .try_get(0)
        .map_err(|e| LdbError::SqlBuild(e.to_string()))?;
    Ok(n as u64)
}

// 从 PostgreSQL 查询行第一列解析 COUNT 标量（`i64` → `u64`）。
pub(crate) fn pg_row_first_u64(row: &sqlx::postgres::PgRow) -> Result<u64, LdbError> {
    use sqlx::Row;
    let n: i64 = row
        .try_get(0)
        .map_err(|e| LdbError::SqlBuild(e.to_string()))?;
    Ok(n as u64)
}

pub(crate) fn mysql_read_column(
    row: &sqlx::mysql::MySqlRow,
    column: &str,
) -> Result<SqlValue, LdbError> {
    use sqlx::Row;
    if let Ok(v) = row.try_get::<Option<i64>, _>(column) {
        return Ok(v.map(SqlValue::I64).unwrap_or(SqlValue::Null));
    }
    if let Ok(v) = row.try_get::<Option<i32>, _>(column) {
        return Ok(v.map(|n| SqlValue::I64(n as i64)).unwrap_or(SqlValue::Null));
    }
    if let Ok(v) = row.try_get::<Option<String>, _>(column) {
        return Ok(v.map(SqlValue::String).unwrap_or(SqlValue::Null));
    }
    if let Ok(v) = row.try_get::<Option<bool>, _>(column) {
        return Ok(v.map(SqlValue::Bool).unwrap_or(SqlValue::Null));
    }
    if let Ok(v) = row.try_get::<Option<f64>, _>(column) {
        return Ok(v.map(SqlValue::F64).unwrap_or(SqlValue::Null));
    }
    Err(LdbError::ModelMapping(format!("无法读取列 `{column}`")))
}

pub(crate) fn pg_read_column(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<SqlValue, LdbError> {
    use sqlx::Row;
    if let Ok(v) = row.try_get::<Option<i64>, _>(column) {
        return Ok(v.map(SqlValue::I64).unwrap_or(SqlValue::Null));
    }
    if let Ok(v) = row.try_get::<Option<i32>, _>(column) {
        return Ok(v.map(|n| SqlValue::I64(n as i64)).unwrap_or(SqlValue::Null));
    }
    if let Ok(v) = row.try_get::<Option<String>, _>(column) {
        return Ok(v.map(SqlValue::String).unwrap_or(SqlValue::Null));
    }
    if let Ok(v) = row.try_get::<Option<bool>, _>(column) {
        return Ok(v.map(SqlValue::Bool).unwrap_or(SqlValue::Null));
    }
    if let Ok(v) = row.try_get::<Option<f64>, _>(column) {
        return Ok(v.map(SqlValue::F64).unwrap_or(SqlValue::Null));
    }
    Err(LdbError::ModelMapping(format!("无法读取列 `{column}`")))
}

fn column_name_list_for<T: LdbModel>() -> Vec<String> {
    T::column_meta_list()
        .iter()
        .map(|m| m.column_name.to_string())
        .collect()
}

async fn mysql_fetch_models<T: LdbModel + Default>(
    pool: &Pool<MySql>,
    built: &BuiltSql,
    dialect: &dyn Dialect,
) -> Result<Vec<T>, LdbError> {
    let sql = crate::sql_build::dialect_exec_sql(dialect, built);
    let mut q = sqlx::query(&sql);
    for v in &built.arg_list {
        q = bind_mysql(q, v);
    }
    let rows = q.fetch_all(pool).await?;
    let column_name_list = column_name_list_for::<T>();
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let column_value_list: Vec<(String, SqlValue)> = column_name_list
            .iter()
            .map(|col| mysql_read_column(&row, col).map(|val| (col.clone(), val)))
            .collect::<Result<_, _>>()?;
        out.push(hydrate_model(&column_value_list)?);
    }
    Ok(out)
}

async fn pg_fetch_models<T: LdbModel + Default>(
    pool: &Pool<Postgres>,
    built: &BuiltSql,
    dialect: &dyn Dialect,
) -> Result<Vec<T>, LdbError> {
    let sql = crate::sql_build::dialect_exec_sql(dialect, built);
    let mut q = sqlx::query(&sql);
    for v in &built.arg_list {
        q = bind_pg(q, v);
    }
    let rows = q.fetch_all(pool).await?;
    let column_name_list = column_name_list_for::<T>();
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let column_value_list: Vec<(String, SqlValue)> = column_name_list
            .iter()
            .map(|col| pg_read_column(&row, col).map(|val| (col.clone(), val)))
            .collect::<Result<_, _>>()?;
        out.push(hydrate_model(&column_value_list)?);
    }
    Ok(out)
}

// MySQL：`dialect_exec_sql` 后 `fetch_one` 读取标量。
async fn mysql_query_scalar_u64(
    pool: &Pool<MySql>,
    built: &BuiltSql,
    dialect: &dyn Dialect,
) -> Result<u64, LdbError> {
    let sql = crate::sql_build::dialect_exec_sql(dialect, built);
    let mut q = sqlx::query(&sql);
    for v in &built.arg_list {
        q = bind_mysql(q, v);
    }
    let row = q.fetch_one(pool).await?;
    mysql_row_first_u64(&row)
}

// MySQL：`dialect_exec_sql` 后 `fetch_optional` 判断行是否存在。
async fn mysql_query_exists(
    pool: &Pool<MySql>,
    built: &BuiltSql,
    dialect: &dyn Dialect,
) -> Result<bool, LdbError> {
    let sql = crate::sql_build::dialect_exec_sql(dialect, built);
    let mut q = sqlx::query(&sql);
    for v in &built.arg_list {
        q = bind_mysql(q, v);
    }
    Ok(q.fetch_optional(pool).await?.is_some())
}

// PostgreSQL：`dialect_exec_sql` 后 `fetch_one` 读取标量。
async fn pg_query_scalar_u64(
    pool: &Pool<Postgres>,
    built: &BuiltSql,
    dialect: &dyn Dialect,
) -> Result<u64, LdbError> {
    let sql = crate::sql_build::dialect_exec_sql(dialect, built);
    let mut q = sqlx::query(&sql);
    for v in &built.arg_list {
        q = bind_pg(q, v);
    }
    let row = q.fetch_one(pool).await?;
    pg_row_first_u64(&row)
}

// PostgreSQL：`dialect_exec_sql` 后 `fetch_optional` 判断行是否存在。
async fn pg_query_exists(
    pool: &Pool<Postgres>,
    built: &BuiltSql,
    dialect: &dyn Dialect,
) -> Result<bool, LdbError> {
    let sql = crate::sql_build::dialect_exec_sql(dialect, built);
    let mut q = sqlx::query(&sql);
    for v in &built.arg_list {
        q = bind_pg(q, v);
    }
    Ok(q.fetch_optional(pool).await?.is_some())
}

impl SqlExecutor for MysqlEngine {
    fn db_kind(&self) -> DbKind {
        DbKind::Mysql
    }

    fn dialect(&self) -> &dyn Dialect {
        self.dialect.as_ref()
    }

    async fn execute_built(&self, built: &BuiltSql) -> Result<u64, LdbError> {
        let sql = crate::sql_build::dialect_exec_sql(self.dialect(), built);
        let mut q = sqlx::query(&sql);
        for v in &built.arg_list {
            q = bind_mysql(q, v);
        }
        Ok(q.execute(&self.pool).await?.rows_affected())
    }

    async fn query_rows(&self, built: &BuiltSql) -> Result<u64, LdbError> {
        let sql = crate::sql_build::dialect_exec_sql(self.dialect(), built);
        let mut q = sqlx::query(&sql);
        for v in &built.arg_list {
            q = bind_mysql(q, v);
        }
        Ok(q.fetch_all(&self.pool).await?.len() as u64)
    }

    async fn fetch_models<T: LdbModel + Default>(
        &self,
        built: &BuiltSql,
    ) -> Result<Vec<T>, LdbError> {
        mysql_fetch_models(&self.pool, built, self.dialect()).await
    }

    async fn query_scalar_u64(&self, built: &BuiltSql) -> Result<u64, LdbError> {
        mysql_query_scalar_u64(&self.pool, built, self.dialect()).await
    }

    async fn query_exists(&self, built: &BuiltSql) -> Result<bool, LdbError> {
        mysql_query_exists(&self.pool, built, self.dialect()).await
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
        let sql = crate::sql_build::dialect_exec_sql(self.dialect(), built);
        let mut q = sqlx::query(&sql);
        for v in &built.arg_list {
            q = bind_pg(q, v);
        }
        Ok(q.execute(&self.pool).await?.rows_affected())
    }

    async fn query_rows(&self, built: &BuiltSql) -> Result<u64, LdbError> {
        let sql = crate::sql_build::dialect_exec_sql(self.dialect(), built);
        let mut q = sqlx::query(&sql);
        for v in &built.arg_list {
            q = bind_pg(q, v);
        }
        Ok(q.fetch_all(&self.pool).await?.len() as u64)
    }

    async fn fetch_models<T: LdbModel + Default>(
        &self,
        built: &BuiltSql,
    ) -> Result<Vec<T>, LdbError> {
        pg_fetch_models(&self.pool, built, self.dialect()).await
    }

    async fn query_scalar_u64(&self, built: &BuiltSql) -> Result<u64, LdbError> {
        pg_query_scalar_u64(&self.pool, built, self.dialect()).await
    }

    async fn query_exists(&self, built: &BuiltSql) -> Result<bool, LdbError> {
        pg_query_exists(&self.pool, built, self.dialect()).await
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

/// 单元测试用：单列名 + 值的行。
pub type MockRow = Vec<(String, SqlValue)>;

/// Mock 执行器：不连接数据库，仅记录 SQL。
#[derive(Debug, Default)]
pub struct MockExecutor {
    pub recorded: Arc<Mutex<RecordedSql>>,
    mock_row_list: Arc<Mutex<Vec<MockRow>>>,
}

impl MockExecutor {
    pub fn last_sql(&self) -> RecordedSql {
        self.recorded.lock().unwrap().clone()
    }

    /// 预设 SELECT 返回的行（列名 + 值）；供 `first` / `list` 单元测试。
    pub fn set_mock_rows(&self, row_list: Vec<MockRow>) {
        *self.mock_row_list.lock().unwrap() = row_list;
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
        Ok(self.mock_row_list.lock().unwrap().len() as u64)
    }

    async fn fetch_models<T: LdbModel + Default>(
        &self,
        built: &BuiltSql,
    ) -> Result<Vec<T>, LdbError> {
        let mut rec = self.recorded.lock().unwrap();
        rec.sql = built.sql.clone();
        rec.arg_list = built.arg_list.clone();
        let row_list = self.mock_row_list.lock().unwrap().clone();
        row_list
            .into_iter()
            .map(|column_value_list| hydrate_model(&column_value_list))
            .collect()
    }

    // 测试桩：固定返回 1，模拟 COUNT 结果。
    async fn query_scalar_u64(&self, built: &BuiltSql) -> Result<u64, LdbError> {
        let mut rec = self.recorded.lock().unwrap();
        rec.sql = built.sql.clone();
        rec.arg_list = built.arg_list.clone();
        Ok(1)
    }

    // 测试桩：SQL 含 `SELECT` 时视为存在匹配行。
    async fn query_exists(&self, built: &BuiltSql) -> Result<bool, LdbError> {
        let mut rec = self.recorded.lock().unwrap();
        rec.sql = built.sql.clone();
        rec.arg_list = built.arg_list.clone();
        Ok(built.sql.contains("SELECT"))
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

    #[tokio::test]
    async fn mock_query_scalar_and_exists() {
        let mock = MockExecutor::default();
        let count_built = crate::sql_build::BuiltSql {
            sql: "SELECT COUNT(*) FROM t".into(),
            arg_list: vec![],
        };
        assert_eq!(mock.query_scalar_u64(&count_built).await.unwrap(), 1);
        assert_eq!(mock.last_sql().sql, "SELECT COUNT(*) FROM t");

        let has_built = crate::sql_build::BuiltSql {
            sql: "SELECT 1 FROM t LIMIT 1".into(),
            arg_list: vec![],
        };
        assert!(mock.query_exists(&has_built).await.unwrap());
        assert_eq!(mock.last_sql().sql, "SELECT 1 FROM t LIMIT 1");
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

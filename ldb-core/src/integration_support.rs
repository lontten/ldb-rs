//! 集成测试辅助（建表、清表、样例数据）。

#![cfg(feature = "integration")]

use crate::Engine;
use crate::error::LdbError;
use crate::exec::{MysqlEngine, PgEngine};
use crate::test_util::TestUser;

pub const MYSQL_DDL: &str = r#"
CREATE TABLE IF NOT EXISTS t_user (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(255) UNIQUE,
    age INT
)
"#;

pub const PG_DDL: &str = r#"
CREATE TABLE IF NOT EXISTS t_user (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) UNIQUE,
    age INT
)
"#;

pub fn mysql_url() -> Option<String> {
    std::env::var("LDB_MYSQL_URL").ok()
}

pub fn pg_url() -> Option<String> {
    std::env::var("LDB_PG_URL").ok()
}

async fn open_mysql_engine(url: &str) -> Result<MysqlEngine, LdbError> {
    let engine = crate::connect_mysql_url(url).await?;
    engine.ping().await?;
    engine.exec_sql(MYSQL_DDL).await?;
    engine.exec_sql("TRUNCATE TABLE t_user").await?;
    Ok(engine)
}

async fn open_pg_engine(url: &str) -> Result<PgEngine, LdbError> {
    let engine = crate::connect_pg_url(url).await?;
    engine.ping().await?;
    engine.exec_sql(PG_DDL).await?;
    engine.exec_sql("TRUNCATE TABLE t_user").await?;
    Ok(engine)
}

/// 集成测试入口：未设置 `LDB_MYSQL_URL` 时由 `#[ignore]` 跳过；已设置但连不上则 panic。
pub async fn require_mysql() -> MysqlEngine {
    let url = mysql_url().unwrap_or_else(|| {
        panic!("LDB_MYSQL_URL must be set (run with --include-ignored)");
    });
    open_mysql_engine(&url).await.unwrap_or_else(|e| {
        panic!("LDB_MYSQL_URL is set but MySQL is unreachable: {e}");
    })
}

/// 集成测试入口：未设置 `LDB_PG_URL` 时由 `#[ignore]` 跳过；已设置但连不上则 panic。
pub async fn require_pg() -> PgEngine {
    let url = pg_url().unwrap_or_else(|| {
        panic!("LDB_PG_URL must be set (run with --include-ignored)");
    });
    open_pg_engine(&url).await.unwrap_or_else(|e| {
        panic!("LDB_PG_URL is set but PostgreSQL is unreachable: {e}");
    })
}

pub fn sample_user(name: &str, age: i32) -> TestUser {
    TestUser {
        id: None,
        name: Some(name.into()),
        age: Some(age),
    }
}

pub async fn reset_mysql_table(engine: &MysqlEngine) -> Result<(), LdbError> {
    engine.exec_sql("TRUNCATE TABLE t_user").await
}

pub async fn reset_pg_table(engine: &PgEngine) -> Result<(), LdbError> {
    engine.exec_sql("TRUNCATE TABLE t_user").await
}

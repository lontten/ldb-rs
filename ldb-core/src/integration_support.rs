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

pub async fn setup_mysql() -> Option<MysqlEngine> {
    let url = mysql_url()?;
    let engine = crate::connect_mysql_url(&url).await.ok()?;
    engine.ping().await.ok()?;
    engine.exec_sql(MYSQL_DDL).await.ok()?;
    engine.exec_sql("TRUNCATE TABLE t_user").await.ok()?;
    Some(engine)
}

pub async fn setup_pg() -> Option<PgEngine> {
    let url = pg_url()?;
    let engine = crate::connect_pg_url(&url).await.ok()?;
    engine.ping().await.ok()?;
    engine.exec_sql(PG_DDL).await.ok()?;
    engine.exec_sql("TRUNCATE TABLE t_user").await.ok()?;
    Some(engine)
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

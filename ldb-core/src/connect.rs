//! 数据库连接入口。

use std::sync::Arc;

use crate::config::{MysqlConfig, PgConfig, PoolConfig};
use crate::error::LdbError;
use crate::exec::{MysqlEngine, PgEngine, open_mysql_pool, open_pg_pool};
use crate::mysql_dialect::MysqlDialect;
use crate::pg_dialect::PgDialect;

/// 建立 MySQL 连接池。
pub async fn connect_mysql(
    config: &MysqlConfig,
    pool: Option<&PoolConfig>,
) -> Result<MysqlEngine, LdbError> {
    let sqlx_pool = open_mysql_pool(config, pool).await?;
    Ok(MysqlEngine {
        pool: sqlx_pool,
        dialect: Arc::new(MysqlDialect),
    })
}

/// 从完整 DSN URL 建立 MySQL 连接池（集成测试与运维脚本用）。
#[cfg(any(feature = "integration", test))]
pub async fn connect_mysql_url(url: &str) -> Result<MysqlEngine, LdbError> {
    let sqlx_pool = sqlx::MySqlPool::connect(url).await?;
    Ok(MysqlEngine {
        pool: sqlx_pool,
        dialect: Arc::new(MysqlDialect),
    })
}

/// 从完整 DSN URL 建立 PostgreSQL 连接池（集成测试与运维脚本用）。
#[cfg(any(feature = "integration", test))]
pub async fn connect_pg_url(url: &str) -> Result<PgEngine, LdbError> {
    let sqlx_pool = sqlx::PgPool::connect(url).await?;
    Ok(PgEngine {
        pool: sqlx_pool,
        dialect: Arc::new(PgDialect),
    })
}

/// 建立 PostgreSQL 连接池。
pub async fn connect_pg(
    config: &PgConfig,
    pool: Option<&PoolConfig>,
) -> Result<PgEngine, LdbError> {
    let sqlx_pool = open_pg_pool(config, pool).await?;
    Ok(PgEngine {
        pool: sqlx_pool,
        dialect: Arc::new(PgDialect),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mysql_dsn_from_config() {
        let cfg = MysqlConfig {
            host: "h".into(),
            port: "3306".into(),
            db_name: "d".into(),
            user: "u".into(),
            password: "p".into(),
            other: String::new(),
            version: crate::config::MysqlVersion::Latest,
        };
        assert!(cfg.dsn().contains("mysql://"));
    }
}

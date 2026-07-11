//! 数据库连接与连接池配置。

use std::time::Duration;

use serde::{Deserialize, Serialize};

/// MySQL 服务端版本枚举，影响 upsert 与主键回填策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MysqlVersion {
    /// 使用运行时检测或默认最新策略。
    Latest,
    /// MySQL 5.x。
    V5,
    /// MySQL 8.0.19。
    V8_0_19,
    /// MySQL 8.0.20 及以上。
    V8_0_20,
}

/// MySQL 连接配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MysqlConfig {
    /// 主机地址。
    pub host: String,
    /// 端口。
    pub port: String,
    /// 数据库名。
    pub db_name: String,
    /// 用户名。
    pub user: String,
    /// 密码。
    pub password: String,
    /// DSN 附加参数（如 `charset=utf8mb4`）。
    pub other: String,
    /// 服务端版本提示。
    pub version: MysqlVersion,
}

/// PostgreSQL 连接配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PgConfig {
    /// 主机地址。
    pub host: String,
    /// 端口。
    pub port: String,
    /// 数据库名。
    pub db_name: String,
    /// 用户名。
    pub user: String,
    /// 密码。
    pub password: String,
    /// DSN 附加参数。
    pub other: String,
}

/// 连接池配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    /// 最大空闲连接数；`None` 表示驱动默认。
    pub max_idle_count: Option<u32>,
    /// 最大打开连接数；`None` 或 `0` 表示不限制。
    pub max_open: Option<u32>,
    /// 连接最大复用时长。
    pub max_lifetime: Option<Duration>,
    /// 连接最大空闲时长。
    pub max_idle_time: Option<Duration>,
}

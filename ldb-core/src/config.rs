//! 数据库连接与连接池配置。

use std::time::Duration;

use serde::{Deserialize, Serialize};

/// MySQL 服务端版本枚举，影响 upsert 与主键回填策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MysqlVersion {
    /// 使用运行时检测或默认最新策略。
    #[default]
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

impl MysqlConfig {
    /// 构建 MySQL 连接 URL。
    pub fn dsn(&self) -> String {
        let base = format!(
            "mysql://{}:{}@{}:{}/{}",
            urlencoding_encode(&self.user),
            urlencoding_encode(&self.password),
            self.host,
            self.port,
            self.db_name
        );
        if self.other.is_empty() {
            base
        } else {
            format!("{base}?{}", self.other)
        }
    }
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

impl PgConfig {
    /// 构建 PostgreSQL 连接 URL。
    pub fn dsn(&self) -> String {
        let base = format!(
            "postgres://{}:{}@{}:{}/{}",
            urlencoding_encode(&self.user),
            urlencoding_encode(&self.password),
            self.host,
            self.port,
            self.db_name
        );
        if self.other.is_empty() {
            base
        } else {
            format!("{base}?{}", self.other)
        }
    }
}

fn urlencoding_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                out.push('%');
                out.push(char::from_digit((b >> 4) as u32, 16).unwrap_or('0'));
                out.push(char::from_digit((b & 0xf) as u32, 16).unwrap_or('0'));
            }
        }
    }
    out
}

/// 连接池配置。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mysql_dsn_basic() {
        let cfg = MysqlConfig {
            host: "127.0.0.1".into(),
            port: "3306".into(),
            db_name: "test".into(),
            user: "root".into(),
            password: "p@ss".into(),
            other: "charset=utf8mb4".into(),
            version: MysqlVersion::Latest,
        };
        let dsn = cfg.dsn();
        assert!(dsn.starts_with("mysql://"));
        assert!(dsn.contains("127.0.0.1:3306/test"));
        assert!(dsn.contains("charset=utf8mb4"));
        assert!(dsn.contains("p%40ss"));
    }

    #[test]
    fn pg_dsn_basic() {
        let cfg = PgConfig {
            host: "127.0.0.1".into(),
            port: "5432".into(),
            db_name: "test".into(),
            user: "postgres".into(),
            password: "123456".into(),
            other: "sslmode=disable".into(),
        };
        let dsn = cfg.dsn();
        assert!(dsn.starts_with("postgres://"));
        assert!(dsn.contains("sslmode=disable"));
    }
}

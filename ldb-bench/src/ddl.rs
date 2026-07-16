//! `t_user` 表 DDL（基准扩展列）。

pub const MYSQL_DROP: &str = "DROP TABLE IF EXISTS t_user";
pub const PG_DROP: &str = "DROP TABLE IF EXISTS t_user";

pub const MYSQL_DDL: &str = r#"
CREATE TABLE t_user (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    age INT NOT NULL,
    status SMALLINT NOT NULL,
    city VARCHAR(64) NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE KEY uk_t_user_name (name)
)
"#;

pub const PG_DDL: &str = r#"
CREATE TABLE t_user (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    age INT NOT NULL,
    status SMALLINT NOT NULL,
    city VARCHAR(64),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (name)
)
"#;

pub const TRUNCATE: &str = "TRUNCATE TABLE t_user";

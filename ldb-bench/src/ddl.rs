//! `t_user` 表 DDL，与 integration 测试一致。

pub const MYSQL_DDL: &str = r#"
CREATE TABLE IF NOT EXISTS t_user (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(255),
    age INT
)
"#;

pub const PG_DDL: &str = r#"
CREATE TABLE IF NOT EXISTS t_user (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255),
    age INT
)
"#;

pub const TRUNCATE: &str = "TRUNCATE TABLE t_user";

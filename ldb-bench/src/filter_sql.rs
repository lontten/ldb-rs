//! sqlx / 类 SQL 后端共用的动态 WHERE 拼装（对齐 `FILTER`）。

use crate::scenario::{FILTER, PAGE};

/// MySQL：WHERE 子句（不含 WHERE 关键字）与按顺序的绑定说明。
pub fn mysql_where_clause() -> (String, Vec<MysqlBind>) {
    let mut parts = Vec::new();
    let mut binds = Vec::new();
    if let Some(pattern) = FILTER.name_like {
        parts.push("name LIKE ?".to_string());
        binds.push(MysqlBind::Str(pattern));
    }
    if let Some(v) = FILTER.age_min {
        parts.push("age >= ?".to_string());
        binds.push(MysqlBind::I32(v));
    }
    if let Some(v) = FILTER.age_max {
        parts.push("age <= ?".to_string());
        binds.push(MysqlBind::I32(v));
    }
    if let Some(v) = FILTER.status {
        parts.push("status = ?".to_string());
        binds.push(MysqlBind::I16(v));
    }
    if let Some(v) = FILTER.city {
        parts.push("city = ?".to_string());
        binds.push(MysqlBind::Str(v));
    }
    (parts.join(" AND "), binds)
}

/// PostgreSQL：WHERE 子句（`$n` 占位）与绑定。
pub fn pg_where_clause(start_index: usize) -> (String, Vec<PgBind>, usize) {
    let mut parts = Vec::new();
    let mut binds = Vec::new();
    let mut idx = start_index;
    if let Some(pattern) = FILTER.name_like {
        parts.push(format!("name LIKE ${idx}"));
        binds.push(PgBind::Str(pattern));
        idx += 1;
    }
    if let Some(v) = FILTER.age_min {
        parts.push(format!("age >= ${idx}"));
        binds.push(PgBind::I32(v));
        idx += 1;
    }
    if let Some(v) = FILTER.age_max {
        parts.push(format!("age <= ${idx}"));
        binds.push(PgBind::I32(v));
        idx += 1;
    }
    if let Some(v) = FILTER.status {
        parts.push(format!("status = ${idx}"));
        binds.push(PgBind::I16(v));
        idx += 1;
    }
    if let Some(v) = FILTER.city {
        parts.push(format!("city = ${idx}"));
        binds.push(PgBind::Str(v));
        idx += 1;
    }
    (parts.join(" AND "), binds, idx)
}

pub fn mysql_page_select_sql() -> (String, Vec<MysqlBind>) {
    let (where_sql, mut binds) = mysql_where_clause();
    let sql = format!(
        "SELECT id, name, age, status, city, created_at FROM t_user WHERE {where_sql} ORDER BY id ASC LIMIT ? OFFSET ?"
    );
    binds.push(MysqlBind::I64(PAGE.limit as i64));
    binds.push(MysqlBind::I64(PAGE.offset as i64));
    (sql, binds)
}

pub fn mysql_count_sql() -> (String, Vec<MysqlBind>) {
    let (where_sql, binds) = mysql_where_clause();
    (
        format!("SELECT COUNT(*) FROM t_user WHERE {where_sql}"),
        binds,
    )
}

pub fn pg_page_select_sql() -> (String, Vec<PgBind>) {
    let (where_sql, mut binds, next) = pg_where_clause(1);
    let sql = format!(
        "SELECT id, name, age, status, city, created_at FROM t_user WHERE {where_sql} ORDER BY id ASC LIMIT ${next} OFFSET ${}",
        next + 1
    );
    binds.push(PgBind::I64(PAGE.limit as i64));
    binds.push(PgBind::I64(PAGE.offset as i64));
    (sql, binds)
}

pub fn pg_count_sql() -> (String, Vec<PgBind>) {
    let (where_sql, binds, _) = pg_where_clause(1);
    (
        format!("SELECT COUNT(*) FROM t_user WHERE {where_sql}"),
        binds,
    )
}

#[derive(Debug, Clone, Copy)]
pub enum MysqlBind {
    Str(&'static str),
    I16(i16),
    I32(i32),
    I64(i64),
}

#[derive(Debug, Clone, Copy)]
pub enum PgBind {
    Str(&'static str),
    I16(i16),
    I32(i32),
    I64(i64),
}

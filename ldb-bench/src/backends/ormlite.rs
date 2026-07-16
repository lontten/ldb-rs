//! Ormlite 后端（动态 SQL，对齐 filter_sql）。

use ormlite::Pool;
use sqlx::Row;

use crate::filter_sql::{
    MysqlBind, PgBind, mysql_count_sql, mysql_page_select_sql, pg_count_sql, pg_page_select_sql,
};
use crate::scenario::{GET_OR_INSERT_NAME, PATCH_AGE, PATCH_CITY, UPSERT_NAME, delete_id_list};
use crate::setup;
use crate::{DbKind, Scenario};

pub async fn run(
    db: DbKind,
    scenario: Scenario,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = db.url().expect("database url");
    match db {
        DbKind::Mysql => run_mysql(&url, scenario).await,
        DbKind::Postgres => run_pg(&url, scenario).await,
    }
}

async fn run_mysql(
    url: &str,
    scenario: Scenario,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool: Pool<sqlx::MySql> = Pool::connect(url).await?;
    match scenario {
        Scenario::FilterPage => {
            let (sql, binds) = mysql_page_select_sql();
            let mut q = sqlx::query(&sql);
            for b in &binds {
                q = match *b {
                    MysqlBind::Str(v) => q.bind(v),
                    MysqlBind::I16(v) => q.bind(v),
                    MysqlBind::I32(v) => q.bind(v),
                    MysqlBind::I64(v) => q.bind(v),
                };
            }
            let _ = q.fetch_all(&pool).await?;
        }
        Scenario::PageWithTotal => {
            let (csql, cbinds) = mysql_count_sql();
            let mut cq = sqlx::query(&csql);
            for b in &cbinds {
                cq = match *b {
                    MysqlBind::Str(v) => cq.bind(v),
                    MysqlBind::I16(v) => cq.bind(v),
                    MysqlBind::I32(v) => cq.bind(v),
                    MysqlBind::I64(v) => cq.bind(v),
                };
            }
            let _: i64 = cq.fetch_one(&pool).await?.try_get(0)?;
            let (sql, binds) = mysql_page_select_sql();
            let mut q = sqlx::query(&sql);
            for b in &binds {
                q = match *b {
                    MysqlBind::Str(v) => q.bind(v),
                    MysqlBind::I16(v) => q.bind(v),
                    MysqlBind::I32(v) => q.bind(v),
                    MysqlBind::I64(v) => q.bind(v),
                };
            }
            let _ = q.fetch_all(&pool).await?;
        }
        Scenario::PartialUpdate => {
            sqlx::query("UPDATE t_user SET age = ?, city = ? WHERE status = ?")
                .bind(PATCH_AGE)
                .bind(PATCH_CITY)
                .bind(1i16)
                .execute(&pool)
                .await?;
        }
        Scenario::Upsert => {
            sqlx::query(
                "INSERT INTO t_user (name, age, status, city) VALUES (?, ?, ?, ?) \
                 ON DUPLICATE KEY UPDATE age = VALUES(age)",
            )
            .bind(UPSERT_NAME)
            .bind(PATCH_AGE)
            .bind(1i16)
            .bind("shanghai")
            .execute(&pool)
            .await?;
        }
        Scenario::DeleteByIds => {
            let ids = delete_id_list();
            let placeholders = vec!["?"; ids.len()].join(", ");
            let sql = format!("DELETE FROM t_user WHERE id IN ({placeholders})");
            let mut q = sqlx::query(&sql);
            for id in ids {
                q = q.bind(id);
            }
            q.execute(&pool).await?;
            setup::restore_deleted_ids(DbKind::Mysql).await?;
        }
        Scenario::GetOrInsert => {
            let row = sqlx::query("SELECT id FROM t_user WHERE name = ?")
                .bind(GET_OR_INSERT_NAME)
                .fetch_optional(&pool)
                .await?;
            if row.is_none() {
                sqlx::query("INSERT INTO t_user (name, age, status, city) VALUES (?, ?, ?, ?)")
                    .bind(GET_OR_INSERT_NAME)
                    .bind(20i32)
                    .bind(1i16)
                    .bind("shanghai")
                    .execute(&pool)
                    .await?;
            }
        }
    }
    pool.close().await;
    Ok(())
}

async fn run_pg(
    url: &str,
    scenario: Scenario,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool: Pool<sqlx::Postgres> = Pool::connect(url).await?;
    match scenario {
        Scenario::FilterPage => {
            let (sql, binds) = pg_page_select_sql();
            let mut q = sqlx::query(&sql);
            for b in &binds {
                q = match *b {
                    PgBind::Str(v) => q.bind(v),
                    PgBind::I16(v) => q.bind(v),
                    PgBind::I32(v) => q.bind(v),
                    PgBind::I64(v) => q.bind(v),
                };
            }
            let _ = q.fetch_all(&pool).await?;
        }
        Scenario::PageWithTotal => {
            let (csql, cbinds) = pg_count_sql();
            let mut cq = sqlx::query(&csql);
            for b in &cbinds {
                cq = match *b {
                    PgBind::Str(v) => cq.bind(v),
                    PgBind::I16(v) => cq.bind(v),
                    PgBind::I32(v) => cq.bind(v),
                    PgBind::I64(v) => cq.bind(v),
                };
            }
            let _: i64 = cq.fetch_one(&pool).await?.try_get(0)?;
            let (sql, binds) = pg_page_select_sql();
            let mut q = sqlx::query(&sql);
            for b in &binds {
                q = match *b {
                    PgBind::Str(v) => q.bind(v),
                    PgBind::I16(v) => q.bind(v),
                    PgBind::I32(v) => q.bind(v),
                    PgBind::I64(v) => q.bind(v),
                };
            }
            let _ = q.fetch_all(&pool).await?;
        }
        Scenario::PartialUpdate => {
            sqlx::query("UPDATE t_user SET age = $1, city = $2 WHERE status = $3")
                .bind(PATCH_AGE)
                .bind(PATCH_CITY)
                .bind(1i16)
                .execute(&pool)
                .await?;
        }
        Scenario::Upsert => {
            sqlx::query(
                "INSERT INTO t_user (name, age, status, city) VALUES ($1, $2, $3, $4) \
                 ON CONFLICT (name) DO UPDATE SET age = EXCLUDED.age",
            )
            .bind(UPSERT_NAME)
            .bind(PATCH_AGE)
            .bind(1i16)
            .bind("shanghai")
            .execute(&pool)
            .await?;
        }
        Scenario::DeleteByIds => {
            let ids = delete_id_list();
            let placeholders: Vec<_> = (1..=ids.len()).map(|i| format!("${i}")).collect();
            let sql = format!(
                "DELETE FROM t_user WHERE id IN ({})",
                placeholders.join(", ")
            );
            let mut q = sqlx::query(&sql);
            for id in ids {
                q = q.bind(id);
            }
            q.execute(&pool).await?;
            setup::restore_deleted_ids(DbKind::Postgres).await?;
        }
        Scenario::GetOrInsert => {
            let row = sqlx::query("SELECT id FROM t_user WHERE name = $1")
                .bind(GET_OR_INSERT_NAME)
                .fetch_optional(&pool)
                .await?;
            if row.is_none() {
                sqlx::query("INSERT INTO t_user (name, age, status, city) VALUES ($1, $2, $3, $4)")
                    .bind(GET_OR_INSERT_NAME)
                    .bind(20i32)
                    .bind(1i16)
                    .bind("shanghai")
                    .execute(&pool)
                    .await?;
            }
        }
    }
    pool.close().await;
    Ok(())
}

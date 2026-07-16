//! Rbatis 后端（动态 SQL，对齐 filter_sql）。

use rbatis::RBatis;
use serde::{Deserialize, Serialize};

use crate::filter_sql::{
    MysqlBind, PgBind, mysql_count_sql, mysql_page_select_sql, pg_count_sql, pg_page_select_sql,
};
use crate::scenario::{GET_OR_INSERT_NAME, PATCH_AGE, PATCH_CITY, UPSERT_NAME, delete_id_list};
use crate::setup;
use crate::{DbKind, Scenario};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct BenchUser {
    id: Option<i64>,
    name: String,
    age: i32,
    status: i16,
    city: Option<String>,
}

async fn rbats(db: DbKind) -> Result<RBatis, Box<dyn std::error::Error + Send + Sync>> {
    let url = db.url().expect("database url");
    let rb = RBatis::new();
    match db {
        DbKind::Mysql => {
            rb.init(rbdc_mysql::MysqlDriver {}, &url)?;
        }
        DbKind::Postgres => {
            rb.init(rbdc_pg::PostgresDriver {}, &url)?;
        }
    }
    Ok(rb)
}

fn mysql_vals(binds: &[MysqlBind]) -> Vec<rbs::Value> {
    binds
        .iter()
        .map(|b| match *b {
            MysqlBind::Str(v) => rbs::value!(v),
            MysqlBind::I16(v) => rbs::value!(v),
            MysqlBind::I32(v) => rbs::value!(v),
            MysqlBind::I64(v) => rbs::value!(v),
        })
        .collect()
}

fn pg_vals(binds: &[PgBind]) -> Vec<rbs::Value> {
    binds
        .iter()
        .map(|b| match *b {
            PgBind::Str(v) => rbs::value!(v),
            PgBind::I16(v) => rbs::value!(v),
            PgBind::I32(v) => rbs::value!(v),
            PgBind::I64(v) => rbs::value!(v),
        })
        .collect()
}

pub async fn run(
    db: DbKind,
    scenario: Scenario,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rb = rbats(db).await?;
    match db {
        DbKind::Mysql => run_mysql(&rb, scenario).await,
        DbKind::Postgres => run_pg(&rb, scenario).await,
    }
}

async fn run_mysql(
    rb: &RBatis,
    scenario: Scenario,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match scenario {
        Scenario::FilterPage => {
            let (sql, binds) = mysql_page_select_sql();
            let _: Vec<BenchUser> = rb.exec_decode(&sql, mysql_vals(&binds)).await?;
        }
        Scenario::PageWithTotal => {
            let (csql, cbinds) = mysql_count_sql();
            let _: i64 = rb.exec_decode(&csql, mysql_vals(&cbinds)).await?;
            let (sql, binds) = mysql_page_select_sql();
            let _: Vec<BenchUser> = rb.exec_decode(&sql, mysql_vals(&binds)).await?;
        }
        Scenario::PartialUpdate => {
            rb.exec(
                "UPDATE t_user SET age = ?, city = ? WHERE status = ?",
                vec![
                    rbs::value!(PATCH_AGE),
                    rbs::value!(PATCH_CITY),
                    rbs::value!(1i16),
                ],
            )
            .await?;
        }
        Scenario::Upsert => {
            rb.exec(
                "INSERT INTO t_user (name, age, status, city) VALUES (?, ?, ?, ?) \
                 ON DUPLICATE KEY UPDATE age = VALUES(age)",
                vec![
                    rbs::value!(UPSERT_NAME),
                    rbs::value!(PATCH_AGE),
                    rbs::value!(1i16),
                    rbs::value!("shanghai"),
                ],
            )
            .await?;
        }
        Scenario::DeleteByIds => {
            let ids = delete_id_list();
            let placeholders = vec!["?"; ids.len()].join(", ");
            let sql = format!("DELETE FROM t_user WHERE id IN ({placeholders})");
            let vals: Vec<_> = ids.iter().map(|id| rbs::value!(*id)).collect();
            rb.exec(&sql, vals).await?;
            setup::restore_deleted_ids(DbKind::Mysql).await?;
        }
        Scenario::GetOrInsert => {
            let rows: Vec<BenchUser> = rb
                .exec_decode(
                    "SELECT id, name, age, status, city FROM t_user WHERE name = ?",
                    vec![rbs::value!(GET_OR_INSERT_NAME)],
                )
                .await?;
            if rows.is_empty() {
                rb.exec(
                    "INSERT INTO t_user (name, age, status, city) VALUES (?, ?, ?, ?)",
                    vec![
                        rbs::value!(GET_OR_INSERT_NAME),
                        rbs::value!(20i32),
                        rbs::value!(1i16),
                        rbs::value!("shanghai"),
                    ],
                )
                .await?;
            }
        }
    }
    Ok(())
}

async fn run_pg(
    rb: &RBatis,
    scenario: Scenario,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match scenario {
        Scenario::FilterPage => {
            let (sql, binds) = pg_page_select_sql();
            let _: Vec<BenchUser> = rb.exec_decode(&sql, pg_vals(&binds)).await?;
        }
        Scenario::PageWithTotal => {
            let (csql, cbinds) = pg_count_sql();
            let _: i64 = rb.exec_decode(&csql, pg_vals(&cbinds)).await?;
            let (sql, binds) = pg_page_select_sql();
            let _: Vec<BenchUser> = rb.exec_decode(&sql, pg_vals(&binds)).await?;
        }
        Scenario::PartialUpdate => {
            rb.exec(
                "UPDATE t_user SET age = $1, city = $2 WHERE status = $3",
                vec![
                    rbs::value!(PATCH_AGE),
                    rbs::value!(PATCH_CITY),
                    rbs::value!(1i16),
                ],
            )
            .await?;
        }
        Scenario::Upsert => {
            rb.exec(
                "INSERT INTO t_user (name, age, status, city) VALUES ($1, $2, $3, $4) \
                 ON CONFLICT (name) DO UPDATE SET age = EXCLUDED.age",
                vec![
                    rbs::value!(UPSERT_NAME),
                    rbs::value!(PATCH_AGE),
                    rbs::value!(1i16),
                    rbs::value!("shanghai"),
                ],
            )
            .await?;
        }
        Scenario::DeleteByIds => {
            let ids = delete_id_list();
            let placeholders: Vec<_> = (1..=ids.len()).map(|i| format!("${i}")).collect();
            let sql = format!(
                "DELETE FROM t_user WHERE id IN ({})",
                placeholders.join(", ")
            );
            let vals: Vec<_> = ids.iter().map(|id| rbs::value!(*id)).collect();
            rb.exec(&sql, vals).await?;
            setup::restore_deleted_ids(DbKind::Postgres).await?;
        }
        Scenario::GetOrInsert => {
            let rows: Vec<BenchUser> = rb
                .exec_decode(
                    "SELECT id, name, age, status, city FROM t_user WHERE name = $1",
                    vec![rbs::value!(GET_OR_INSERT_NAME)],
                )
                .await?;
            if rows.is_empty() {
                rb.exec(
                    "INSERT INTO t_user (name, age, status, city) VALUES ($1, $2, $3, $4)",
                    vec![
                        rbs::value!(GET_OR_INSERT_NAME),
                        rbs::value!(20i32),
                        rbs::value!(1i16),
                        rbs::value!("shanghai"),
                    ],
                )
                .await?;
            }
        }
    }
    Ok(())
}

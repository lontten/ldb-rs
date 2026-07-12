//! Rbatis 后端。

use rbatis::RBatis;
use serde::{Deserialize, Serialize};

use super::{reset_and_seed, reset_empty};
use crate::{CrudOp, DbKind};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct BenchUser {
    id: Option<i64>,
    name: String,
    age: i32,
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

pub async fn run(
    db: DbKind,
    op: CrudOp,
    n: usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rb = rbats(db).await?;
    match op {
        CrudOp::Insert => {
            reset_empty(db).await?;
            for i in 0..n {
                let sql = match db {
                    DbKind::Mysql => "INSERT INTO t_user (name, age) VALUES (?, ?)",
                    DbKind::Postgres => "INSERT INTO t_user (name, age) VALUES ($1, $2)",
                };
                rb.exec(
                    sql,
                    vec![
                        rbs::value! {"name": format!("user_{i}")},
                        rbs::value! {"age": i as i32},
                    ],
                )
                .await?;
            }
        }
        CrudOp::Update => {
            reset_and_seed(db, n).await?;
            let sql = match db {
                DbKind::Mysql => "UPDATE t_user SET age = ? WHERE id > 0",
                DbKind::Postgres => "UPDATE t_user SET age = $1 WHERE id > 0",
            };
            rb.exec(sql, vec![rbs::value! {"age": 99i32}]).await?;
        }
        CrudOp::Delete => {
            reset_and_seed(db, n).await?;
            rb.exec("DELETE FROM t_user WHERE id > 0", vec![]).await?;
        }
        CrudOp::First => {
            reset_and_seed(db, n).await?;
            let _: Option<BenchUser> = rb
                .exec_decode(
                    "SELECT id, name, age FROM t_user WHERE id > 0 LIMIT 1",
                    vec![],
                )
                .await?;
        }
        CrudOp::List => {
            reset_and_seed(db, n).await?;
            let sql = match db {
                DbKind::Mysql => "SELECT id, name, age FROM t_user WHERE id > 0 LIMIT ?",
                DbKind::Postgres => "SELECT id, name, age FROM t_user WHERE id > 0 LIMIT $1",
            };
            let _: Vec<BenchUser> = rb
                .exec_decode(sql, vec![rbs::value! {"limit": n as i64}])
                .await?;
        }
        CrudOp::Count => {
            reset_and_seed(db, n).await?;
            let _: Option<i64> = rb
                .exec_decode("SELECT COUNT(*) FROM t_user WHERE id > 0", vec![])
                .await?;
        }
    }
    Ok(())
}

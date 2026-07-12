//! sqlx 原生 SQL 后端。

use super::{reset_and_seed, reset_empty};
use crate::{CrudOp, DbKind};

pub async fn run(
    db: DbKind,
    op: CrudOp,
    n: usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match db {
        DbKind::Mysql => run_mysql(op, n).await,
        DbKind::Postgres => run_pg(op, n).await,
    }
}

async fn run_mysql(op: CrudOp, n: usize) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = DbKind::Mysql.url().expect("LDB_MYSQL_URL");
    let pool = sqlx::MySqlPool::connect(&url).await?;
    match op {
        CrudOp::Insert => {
            reset_empty(DbKind::Mysql).await?;
            for i in 0..n {
                sqlx::query("INSERT INTO t_user (name, age) VALUES (?, ?)")
                    .bind(format!("user_{i}"))
                    .bind(i as i32)
                    .execute(&pool)
                    .await?;
            }
        }
        CrudOp::Update => {
            reset_and_seed(DbKind::Mysql, n).await?;
            sqlx::query("UPDATE t_user SET age = ? WHERE id > 0")
                .bind(99i32)
                .execute(&pool)
                .await?;
        }
        CrudOp::Delete => {
            reset_and_seed(DbKind::Mysql, n).await?;
            sqlx::query("DELETE FROM t_user WHERE id > 0")
                .execute(&pool)
                .await?;
        }
        CrudOp::First => {
            reset_and_seed(DbKind::Mysql, n).await?;
            let _: (i64, String, i32) =
                sqlx::query_as("SELECT id, name, age FROM t_user WHERE id > 0 LIMIT 1")
                    .fetch_one(&pool)
                    .await?;
        }
        CrudOp::List => {
            reset_and_seed(DbKind::Mysql, n).await?;
            let _ = sqlx::query_as::<_, (i64, String, i32)>(
                "SELECT id, name, age FROM t_user WHERE id > 0 LIMIT ?",
            )
            .bind(n as i64)
            .fetch_all(&pool)
            .await?;
        }
        CrudOp::Count => {
            reset_and_seed(DbKind::Mysql, n).await?;
            let _: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM t_user WHERE id > 0")
                .fetch_one(&pool)
                .await?;
        }
    }
    pool.close().await;
    Ok(())
}

async fn run_pg(op: CrudOp, n: usize) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = DbKind::Postgres.url().expect("LDB_PG_URL");
    let pool = sqlx::PgPool::connect(&url).await?;
    match op {
        CrudOp::Insert => {
            reset_empty(DbKind::Postgres).await?;
            for i in 0..n {
                sqlx::query("INSERT INTO t_user (name, age) VALUES ($1, $2)")
                    .bind(format!("user_{i}"))
                    .bind(i as i32)
                    .execute(&pool)
                    .await?;
            }
        }
        CrudOp::Update => {
            reset_and_seed(DbKind::Postgres, n).await?;
            sqlx::query("UPDATE t_user SET age = $1 WHERE id > 0")
                .bind(99i32)
                .execute(&pool)
                .await?;
        }
        CrudOp::Delete => {
            reset_and_seed(DbKind::Postgres, n).await?;
            sqlx::query("DELETE FROM t_user WHERE id > 0")
                .execute(&pool)
                .await?;
        }
        CrudOp::First => {
            reset_and_seed(DbKind::Postgres, n).await?;
            let _: (i64, String, i32) =
                sqlx::query_as("SELECT id, name, age FROM t_user WHERE id > 0 LIMIT 1")
                    .fetch_one(&pool)
                    .await?;
        }
        CrudOp::List => {
            reset_and_seed(DbKind::Postgres, n).await?;
            let _ = sqlx::query_as::<_, (i64, String, i32)>(
                "SELECT id, name, age FROM t_user WHERE id > 0 LIMIT $1",
            )
            .bind(n as i64)
            .fetch_all(&pool)
            .await?;
        }
        CrudOp::Count => {
            reset_and_seed(DbKind::Postgres, n).await?;
            let _: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM t_user WHERE id > 0")
                .fetch_one(&pool)
                .await?;
        }
    }
    pool.close().await;
    Ok(())
}

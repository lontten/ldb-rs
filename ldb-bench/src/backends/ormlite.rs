//! Ormlite 后端。

use ormlite::Pool;
use ormlite::model::*;

use super::{reset_and_seed, reset_empty};
use crate::{CrudOp, DbKind};

mod pg_model {
    use super::*;

    #[derive(Model, Debug)]
    #[ormlite(table = "t_user", database = "postgres", insertable = InsertBenchUser)]
    pub(super) struct BenchUser {
        id: i64,
        name: String,
        age: i32,
    }
}

pub async fn run(
    db: DbKind,
    op: CrudOp,
    n: usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = db.url().expect("database url");
    match db {
        DbKind::Mysql => run_mysql(&url, db, op, n).await,
        DbKind::Postgres => run_pg(&url, db, op, n).await,
    }
}

async fn run_mysql(
    url: &str,
    db: DbKind,
    op: CrudOp,
    n: usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool: Pool<sqlx::MySql> = Pool::connect(url).await?;
    match op {
        CrudOp::Insert => {
            reset_empty(db).await?;
            for i in 0..n {
                ormlite::query("INSERT INTO t_user (name, age) VALUES (?, ?)")
                    .bind(format!("user_{i}"))
                    .bind(i as i32)
                    .execute(&pool)
                    .await?;
            }
        }
        CrudOp::Update => {
            reset_and_seed(db, n).await?;
            ormlite::query("UPDATE t_user SET age = ? WHERE id > 0")
                .bind(99i32)
                .execute(&pool)
                .await?;
        }
        CrudOp::Delete => {
            reset_and_seed(db, n).await?;
            ormlite::query("DELETE FROM t_user WHERE id > 0")
                .execute(&pool)
                .await?;
        }
        CrudOp::First => {
            reset_and_seed(db, n).await?;
            let _: (i64, String, i32) =
                ormlite::query_as("SELECT id, name, age FROM t_user WHERE id > 0 LIMIT 1")
                    .fetch_one(&pool)
                    .await?;
        }
        CrudOp::List => {
            reset_and_seed(db, n).await?;
            let _ = ormlite::query_as::<_, (i64, String, i32)>(
                "SELECT id, name, age FROM t_user WHERE id > 0 LIMIT ?",
            )
            .bind(n as i64)
            .fetch_all(&pool)
            .await?;
        }
        CrudOp::Count => {
            reset_and_seed(db, n).await?;
            let _: (i64,) = ormlite::query_as("SELECT COUNT(*) FROM t_user WHERE id > 0")
                .fetch_one(&pool)
                .await?;
        }
    }
    pool.close().await;
    Ok(())
}

async fn run_pg(
    url: &str,
    db: DbKind,
    op: CrudOp,
    n: usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use pg_model::{BenchUser, InsertBenchUser};

    let pool: Pool<sqlx::Postgres> = Pool::connect(url).await?;
    match op {
        CrudOp::Insert => {
            reset_empty(db).await?;
            for i in 0..n {
                InsertBenchUser {
                    name: format!("user_{i}"),
                    age: i as i32,
                }
                .insert(&pool)
                .await?;
            }
        }
        CrudOp::Update => {
            reset_and_seed(db, n).await?;
            ormlite::query("UPDATE t_user SET age = $1 WHERE id > 0")
                .bind(99i32)
                .execute(&pool)
                .await?;
        }
        CrudOp::Delete => {
            reset_and_seed(db, n).await?;
            ormlite::query("DELETE FROM t_user WHERE id > 0")
                .execute(&pool)
                .await?;
        }
        CrudOp::First => {
            reset_and_seed(db, n).await?;
            let _ = BenchUser::select()
                .where_("id > 0")
                .limit(1)
                .fetch_one(&pool)
                .await?;
        }
        CrudOp::List => {
            reset_and_seed(db, n).await?;
            let _ = BenchUser::select()
                .where_("id > 0")
                .limit(n)
                .fetch_all(&pool)
                .await?;
        }
        CrudOp::Count => {
            reset_and_seed(db, n).await?;
            let _: (i64,) = ormlite::query_as("SELECT COUNT(*) FROM t_user WHERE id > 0")
                .fetch_one(&pool)
                .await?;
        }
    }
    pool.close().await;
    Ok(())
}

//! 各 ORM 后端实现。

mod diesel;
mod ldb;
mod ormlite;
mod rbatis;
mod seaorm;
mod sqlx_backend;
mod welds;

use crate::{CrudOp, DbKind, OrmKind};

pub async fn run(
    db: DbKind,
    orm: OrmKind,
    op: CrudOp,
    n: usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match orm {
        OrmKind::Ldb => ldb::run(db, op, n).await,
        OrmKind::Sqlx => sqlx_backend::run(db, op, n).await,
        OrmKind::Seaorm => seaorm::run(db, op, n).await,
        OrmKind::Diesel => diesel::run(db, op, n).await,
        OrmKind::Welds => welds::run(db, op, n).await,
        OrmKind::Rbatis => rbatis::run(db, op, n).await,
        OrmKind::Ormlite => ormlite::run(db, op, n).await,
    }
}

/// 用 sqlx 建表、清表并预置 `n` 行（供读/写类操作共用）。
pub(crate) async fn reset_and_seed(
    db: DbKind,
    n: usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = db.url().expect("database url");
    match db {
        DbKind::Mysql => {
            let pool = ::sqlx::MySqlPool::connect(&url).await?;
            ::sqlx::query(crate::ddl::MYSQL_DDL).execute(&pool).await?;
            ::sqlx::query(crate::ddl::TRUNCATE).execute(&pool).await?;
            for i in 0..n {
                ::sqlx::query("INSERT INTO t_user (name, age) VALUES (?, ?)")
                    .bind(format!("seed_{i}"))
                    .bind(i as i32)
                    .execute(&pool)
                    .await?;
            }
            pool.close().await;
        }
        DbKind::Postgres => {
            let pool = ::sqlx::PgPool::connect(&url).await?;
            ::sqlx::query(crate::ddl::PG_DDL).execute(&pool).await?;
            ::sqlx::query(crate::ddl::TRUNCATE).execute(&pool).await?;
            for i in 0..n {
                ::sqlx::query("INSERT INTO t_user (name, age) VALUES ($1, $2)")
                    .bind(format!("seed_{i}"))
                    .bind(i as i32)
                    .execute(&pool)
                    .await?;
            }
            pool.close().await;
        }
    }
    Ok(())
}

/// 仅建表并清表。
pub(crate) async fn reset_empty(
    db: DbKind,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = db.url().expect("database url");
    match db {
        DbKind::Mysql => {
            let pool = ::sqlx::MySqlPool::connect(&url).await?;
            ::sqlx::query(crate::ddl::MYSQL_DDL).execute(&pool).await?;
            ::sqlx::query(crate::ddl::TRUNCATE).execute(&pool).await?;
            pool.close().await;
        }
        DbKind::Postgres => {
            let pool = ::sqlx::PgPool::connect(&url).await?;
            ::sqlx::query(crate::ddl::PG_DDL).execute(&pool).await?;
            ::sqlx::query(crate::ddl::TRUNCATE).execute(&pool).await?;
            pool.close().await;
        }
    }
    Ok(())
}

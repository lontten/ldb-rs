//! 测量外建表与种子数据。

use crate::scenario::{SEED_N, SeedRow, delete_id_list, seed_row};
use crate::{DbKind, ddl};

/// 重建表并写入固定种子；每个 DB 在 bench 开始时调用一次。
pub async fn prepare(db: DbKind) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = db.url().ok_or("database url missing")?;
    match db {
        DbKind::Mysql => {
            let pool = sqlx::MySqlPool::connect(&url).await?;
            sqlx::query(ddl::MYSQL_DROP).execute(&pool).await?;
            sqlx::query(ddl::MYSQL_DDL).execute(&pool).await?;
            for i in 0..SEED_N {
                let row = seed_row(i);
                sqlx::query("INSERT INTO t_user (name, age, status, city) VALUES (?, ?, ?, ?)")
                    .bind(&row.name)
                    .bind(row.age)
                    .bind(row.status)
                    .bind(&row.city)
                    .execute(&pool)
                    .await?;
            }
            pool.close().await;
        }
        DbKind::Postgres => {
            let pool = sqlx::PgPool::connect(&url).await?;
            sqlx::query(ddl::PG_DROP).execute(&pool).await?;
            sqlx::query(ddl::PG_DDL).execute(&pool).await?;
            for i in 0..SEED_N {
                let row = seed_row(i);
                sqlx::query("INSERT INTO t_user (name, age, status, city) VALUES ($1, $2, $3, $4)")
                    .bind(&row.name)
                    .bind(row.age)
                    .bind(row.status)
                    .bind(&row.city)
                    .execute(&pool)
                    .await?;
            }
            pool.close().await;
        }
    }
    Ok(())
}

/// `delete_by_ids` 后恢复被删行（显式 id，保持后续迭代稳定）。
pub async fn restore_deleted_ids(
    db: DbKind,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = db.url().ok_or("database url missing")?;
    let id_list = delete_id_list();
    match db {
        DbKind::Mysql => {
            let pool = sqlx::MySqlPool::connect(&url).await?;
            for id in id_list {
                let row = seed_row((id - 1) as usize);
                insert_mysql(&pool, id, &row).await?;
            }
            pool.close().await;
        }
        DbKind::Postgres => {
            let pool = sqlx::PgPool::connect(&url).await?;
            for id in id_list {
                let row = seed_row((id - 1) as usize);
                insert_pg(&pool, id, &row).await?;
            }
            pool.close().await;
        }
    }
    Ok(())
}

async fn insert_mysql(
    pool: &sqlx::MySqlPool,
    id: i64,
    row: &SeedRow,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sqlx::query("INSERT INTO t_user (id, name, age, status, city) VALUES (?, ?, ?, ?, ?)")
        .bind(id)
        .bind(&row.name)
        .bind(row.age)
        .bind(row.status)
        .bind(&row.city)
        .execute(pool)
        .await?;
    Ok(())
}

async fn insert_pg(
    pool: &sqlx::PgPool,
    id: i64,
    row: &SeedRow,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sqlx::query("INSERT INTO t_user (id, name, age, status, city) VALUES ($1, $2, $3, $4, $5)")
        .bind(id)
        .bind(&row.name)
        .bind(row.age)
        .bind(row.status)
        .bind(&row.city)
        .execute(pool)
        .await?;
    Ok(())
}

//! ldb 后端。

use ldb_core::{
    Engine, connect_mysql_url, connect_pg_url, count, delete, first, insert, list,
    test_util::TestUser, update, w,
};

use super::{reset_and_seed, reset_empty};
use crate::{CrudOp, DbKind};

pub async fn run(
    db: DbKind,
    op: CrudOp,
    n: usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match db {
        DbKind::Mysql => {
            let url = db.url().expect("LDB_MYSQL_URL");
            let engine = connect_mysql_url(&url).await?;
            run_with_engine(&engine, db, op, n).await
        }
        DbKind::Postgres => {
            let url = db.url().expect("LDB_PG_URL");
            let engine = connect_pg_url(&url).await?;
            run_with_engine(&engine, db, op, n).await
        }
    }
}

async fn run_with_engine<E: Engine>(
    engine: &E,
    db: DbKind,
    op: CrudOp,
    n: usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match op {
        CrudOp::Insert => {
            reset_empty(db).await?;
            for i in 0..n {
                let mut user = TestUser {
                    id: None,
                    name: Some(format!("user_{i}")),
                    age: Some(i as i32),
                };
                insert(engine, &mut user).await?;
            }
        }
        CrudOp::Update => {
            reset_and_seed(db, n).await?;
            let patch = TestUser {
                id: None,
                name: None,
                age: Some(99),
            };
            update(engine, &patch).where_(w().gt("id", 0)).await?;
        }
        CrudOp::Delete => {
            reset_and_seed(db, n).await?;
            delete::<TestUser, _>(engine)
                .where_(w().gt("id", 0))
                .await?;
        }
        CrudOp::First => {
            reset_and_seed(db, n).await?;
            let _ = first::<TestUser, _>(engine).where_(w().gt("id", 0)).await?;
        }
        CrudOp::List => {
            reset_and_seed(db, n).await?;
            let _ = list::<TestUser, _>(engine)
                .where_(w().gt("id", 0))
                .limit(n as u64)
                .await?;
        }
        CrudOp::Count => {
            reset_and_seed(db, n).await?;
            let _ = count::<TestUser, _>(engine).where_(w().gt("id", 0)).await?;
        }
    }
    Ok(())
}

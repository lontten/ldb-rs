//! diesel-async 后端。

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel_async::pooled_connection::{AsyncDieselConnectionManager, deadpool::Pool};

use super::{reset_and_seed, reset_empty};
use crate::schema_diesel::t_user;
use crate::schema_diesel::t_user::dsl::*;
use crate::{CrudOp, DbKind};

#[derive(Insertable)]
#[diesel(table_name = t_user)]
struct NewUser {
    name: Option<String>,
    age: Option<i32>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = t_user)]
struct UserRow {
    #[allow(dead_code)]
    id: i64,
    #[allow(dead_code)]
    name: Option<String>,
    #[allow(dead_code)]
    age: Option<i32>,
}

type DieselMysqlPool = Pool<diesel_async::AsyncMysqlConnection>;
type DieselPgPool = Pool<diesel_async::AsyncPgConnection>;

async fn mysql_conn() -> Result<DieselMysqlPool, Box<dyn std::error::Error + Send + Sync>> {
    let url = DbKind::Mysql.url().expect("LDB_MYSQL_URL");
    let manager = AsyncDieselConnectionManager::<diesel_async::AsyncMysqlConnection>::new(url);
    Ok(Pool::builder(manager).build()?)
}

async fn pg_conn() -> Result<DieselPgPool, Box<dyn std::error::Error + Send + Sync>> {
    let url = DbKind::Postgres.url().expect("LDB_PG_URL");
    let manager = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(url);
    Ok(Pool::builder(manager).build()?)
}

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
    let pool = mysql_conn().await?;
    let mut conn = pool.get().await?;
    match op {
        CrudOp::Insert => {
            reset_empty(DbKind::Mysql).await?;
            for i in 0..n {
                diesel::insert_into(t_user)
                    .values(NewUser {
                        name: Some(format!("user_{i}")),
                        age: Some(i as i32),
                    })
                    .execute(&mut conn)
                    .await?;
            }
        }
        CrudOp::Update => {
            reset_and_seed(DbKind::Mysql, n).await?;
            diesel::update(t_user.filter(id.gt(0)))
                .set(age.eq(99))
                .execute(&mut conn)
                .await?;
        }
        CrudOp::Delete => {
            reset_and_seed(DbKind::Mysql, n).await?;
            diesel::delete(t_user.filter(id.gt(0)))
                .execute(&mut conn)
                .await?;
        }
        CrudOp::First => {
            reset_and_seed(DbKind::Mysql, n).await?;
            let _: UserRow = t_user
                .filter(id.gt(0))
                .select(UserRow::as_select())
                .first(&mut conn)
                .await?;
        }
        CrudOp::List => {
            reset_and_seed(DbKind::Mysql, n).await?;
            let _: Vec<UserRow> = t_user
                .filter(id.gt(0))
                .select(UserRow::as_select())
                .limit(n as i64)
                .load(&mut conn)
                .await?;
        }
        CrudOp::Count => {
            reset_and_seed(DbKind::Mysql, n).await?;
            let _: i64 = t_user
                .filter(id.gt(0))
                .select(diesel::dsl::count_star())
                .first(&mut conn)
                .await?;
        }
    }
    Ok(())
}

async fn run_pg(op: CrudOp, n: usize) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool = pg_conn().await?;
    let mut conn = pool.get().await?;
    match op {
        CrudOp::Insert => {
            reset_empty(DbKind::Postgres).await?;
            for i in 0..n {
                diesel::insert_into(t_user)
                    .values(NewUser {
                        name: Some(format!("user_{i}")),
                        age: Some(i as i32),
                    })
                    .execute(&mut conn)
                    .await?;
            }
        }
        CrudOp::Update => {
            reset_and_seed(DbKind::Postgres, n).await?;
            diesel::update(t_user.filter(id.gt(0)))
                .set(age.eq(99))
                .execute(&mut conn)
                .await?;
        }
        CrudOp::Delete => {
            reset_and_seed(DbKind::Postgres, n).await?;
            diesel::delete(t_user.filter(id.gt(0)))
                .execute(&mut conn)
                .await?;
        }
        CrudOp::First => {
            reset_and_seed(DbKind::Postgres, n).await?;
            let _: UserRow = t_user
                .filter(id.gt(0))
                .select(UserRow::as_select())
                .first(&mut conn)
                .await?;
        }
        CrudOp::List => {
            reset_and_seed(DbKind::Postgres, n).await?;
            let _: Vec<UserRow> = t_user
                .filter(id.gt(0))
                .select(UserRow::as_select())
                .limit(n as i64)
                .load(&mut conn)
                .await?;
        }
        CrudOp::Count => {
            reset_and_seed(DbKind::Postgres, n).await?;
            let _: i64 = t_user
                .filter(id.gt(0))
                .select(diesel::dsl::count_star())
                .first(&mut conn)
                .await?;
        }
    }
    Ok(())
}

//! diesel-async 后端。

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel_async::pooled_connection::{AsyncDieselConnectionManager, deadpool::Pool};

use crate::scenario::{
    FILTER, GET_OR_INSERT_NAME, PAGE, PATCH_AGE, PATCH_CITY, UPSERT_NAME, delete_id_list,
};
use crate::schema_diesel::t_user;
use crate::schema_diesel::t_user::dsl::*;
use crate::setup;
use crate::{DbKind, Scenario};

#[derive(Insertable)]
#[diesel(table_name = t_user)]
struct NewUser {
    name: String,
    age: i32,
    status: i16,
    city: Option<String>,
}

#[derive(AsChangeset)]
#[diesel(table_name = t_user)]
struct PatchUser {
    age: i32,
    city: Option<String>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = t_user)]
struct UserRow {
    #[allow(dead_code)]
    id: i64,
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    age: i32,
    #[allow(dead_code)]
    status: i16,
    #[allow(dead_code)]
    city: Option<String>,
    #[allow(dead_code)]
    created_at: chrono::NaiveDateTime,
}

type DieselMysqlPool = Pool<diesel_async::AsyncMysqlConnection>;
type DieselPgPool = Pool<diesel_async::AsyncPgConnection>;

async fn mysql_pool() -> Result<DieselMysqlPool, Box<dyn std::error::Error + Send + Sync>> {
    let url = DbKind::Mysql.url().expect("LDB_MYSQL_URL");
    let manager = AsyncDieselConnectionManager::<diesel_async::AsyncMysqlConnection>::new(url);
    Ok(Pool::builder(manager).build()?)
}

async fn pg_pool() -> Result<DieselPgPool, Box<dyn std::error::Error + Send + Sync>> {
    let url = DbKind::Postgres.url().expect("LDB_PG_URL");
    let manager = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(url);
    Ok(Pool::builder(manager).build()?)
}

/// 动态叠加 FILTER 条件（boxed 以便按 Option 组装）。
macro_rules! apply_filter {
    ($q:expr) => {{
        let mut q = $q.into_boxed();
        if let Some(pattern) = FILTER.name_like {
            q = q.filter(name.like(pattern));
        }
        if let Some(v) = FILTER.age_min {
            q = q.filter(age.ge(v));
        }
        if let Some(v) = FILTER.age_max {
            q = q.filter(age.le(v));
        }
        if let Some(v) = FILTER.status {
            q = q.filter(status.eq(v));
        }
        if let Some(v) = FILTER.city {
            q = q.filter(city.eq(v));
        }
        q
    }};
}

pub async fn run(
    db: DbKind,
    scenario: Scenario,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match db {
        DbKind::Mysql => run_mysql(scenario).await,
        DbKind::Postgres => run_pg(scenario).await,
    }
}

async fn run_mysql(scenario: Scenario) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool = mysql_pool().await?;
    let mut conn = pool.get().await?;
    match scenario {
        Scenario::FilterPage => {
            let _: Vec<UserRow> = apply_filter!(t_user)
                .select(UserRow::as_select())
                .order(id.asc())
                .limit(PAGE.limit as i64)
                .offset(PAGE.offset as i64)
                .load(&mut conn)
                .await?;
        }
        Scenario::PageWithTotal => {
            let _: i64 = apply_filter!(t_user)
                .select(diesel::dsl::count_star())
                .first(&mut conn)
                .await?;
            let _: Vec<UserRow> = apply_filter!(t_user)
                .select(UserRow::as_select())
                .order(id.asc())
                .limit(PAGE.limit as i64)
                .offset(PAGE.offset as i64)
                .load(&mut conn)
                .await?;
        }
        Scenario::PartialUpdate => {
            diesel::update(t_user.filter(status.eq(1i16)))
                .set(PatchUser {
                    age: PATCH_AGE,
                    city: Some(PATCH_CITY.to_string()),
                })
                .execute(&mut conn)
                .await?;
        }
        Scenario::Upsert => {
            diesel::insert_into(t_user)
                .values(NewUser {
                    name: UPSERT_NAME.to_string(),
                    age: PATCH_AGE,
                    status: 1,
                    city: Some("shanghai".to_string()),
                })
                .on_conflict(diesel::dsl::DuplicatedKeys)
                .do_update()
                .set(age.eq(PATCH_AGE))
                .execute(&mut conn)
                .await?;
        }
        Scenario::DeleteByIds => {
            let ids = delete_id_list();
            diesel::delete(t_user.filter(id.eq_any(ids.as_slice())))
                .execute(&mut conn)
                .await?;
            setup::restore_deleted_ids(DbKind::Mysql).await?;
        }
        Scenario::GetOrInsert => {
            let existing: Option<UserRow> = t_user
                .filter(name.eq(GET_OR_INSERT_NAME))
                .select(UserRow::as_select())
                .first(&mut conn)
                .await
                .optional()?;
            if existing.is_none() {
                diesel::insert_into(t_user)
                    .values(NewUser {
                        name: GET_OR_INSERT_NAME.to_string(),
                        age: 20,
                        status: 1,
                        city: Some("shanghai".to_string()),
                    })
                    .execute(&mut conn)
                    .await?;
            }
        }
    }
    Ok(())
}

async fn run_pg(scenario: Scenario) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool = pg_pool().await?;
    let mut conn = pool.get().await?;
    match scenario {
        Scenario::FilterPage => {
            let _: Vec<UserRow> = apply_filter!(t_user)
                .select(UserRow::as_select())
                .order(id.asc())
                .limit(PAGE.limit as i64)
                .offset(PAGE.offset as i64)
                .load(&mut conn)
                .await?;
        }
        Scenario::PageWithTotal => {
            let _: i64 = apply_filter!(t_user)
                .select(diesel::dsl::count_star())
                .first(&mut conn)
                .await?;
            let _: Vec<UserRow> = apply_filter!(t_user)
                .select(UserRow::as_select())
                .order(id.asc())
                .limit(PAGE.limit as i64)
                .offset(PAGE.offset as i64)
                .load(&mut conn)
                .await?;
        }
        Scenario::PartialUpdate => {
            diesel::update(t_user.filter(status.eq(1i16)))
                .set(PatchUser {
                    age: PATCH_AGE,
                    city: Some(PATCH_CITY.to_string()),
                })
                .execute(&mut conn)
                .await?;
        }
        Scenario::Upsert => {
            diesel::insert_into(t_user)
                .values(NewUser {
                    name: UPSERT_NAME.to_string(),
                    age: PATCH_AGE,
                    status: 1,
                    city: Some("shanghai".to_string()),
                })
                .on_conflict(name)
                .do_update()
                .set(age.eq(PATCH_AGE))
                .execute(&mut conn)
                .await?;
        }
        Scenario::DeleteByIds => {
            let ids = delete_id_list();
            diesel::delete(t_user.filter(id.eq_any(ids.as_slice())))
                .execute(&mut conn)
                .await?;
            setup::restore_deleted_ids(DbKind::Postgres).await?;
        }
        Scenario::GetOrInsert => {
            let existing: Option<UserRow> = t_user
                .filter(name.eq(GET_OR_INSERT_NAME))
                .select(UserRow::as_select())
                .first(&mut conn)
                .await
                .optional()?;
            if existing.is_none() {
                diesel::insert_into(t_user)
                    .values(NewUser {
                        name: GET_OR_INSERT_NAME.to_string(),
                        age: 20,
                        status: 1,
                        city: Some("shanghai".to_string()),
                    })
                    .execute(&mut conn)
                    .await?;
            }
        }
    }
    Ok(())
}

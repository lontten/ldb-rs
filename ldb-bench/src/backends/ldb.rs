//! ldb 后端。

use ldb_core::{
    Engine, OnConflict, Order, connect_mysql_url, connect_pg_url, count, delete, get_or_insert,
    insert, list, update, w,
};

use crate::model::BenchUser;
use crate::scenario::{
    FILTER, GET_OR_INSERT_NAME, PAGE, PATCH_AGE, PATCH_CITY, UPSERT_NAME, delete_id_list,
};
use crate::setup;
use crate::{DbKind, Scenario};

/// 按固定 `FILTER` 构造 WhereBuilder（`None` 字段不进条件）。
pub fn filter_where() -> ldb_core::WhereBuilder {
    let mut wb = w();
    if let Some(pattern) = FILTER.name_like {
        wb = wb.like("name", pattern);
    }
    if let Some(v) = FILTER.age_min {
        wb = wb.gte("age", v);
    }
    if let Some(v) = FILTER.age_max {
        wb = wb.lte("age", v);
    }
    wb = wb.eq_if(
        "status",
        FILTER.status.unwrap_or(0),
        FILTER.status.is_some(),
    );
    wb = wb.eq_if("city", FILTER.city.unwrap_or(""), FILTER.city.is_some());
    wb
}

pub async fn run(
    db: DbKind,
    scenario: Scenario,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match db {
        DbKind::Mysql => {
            let url = db.url().expect("LDB_MYSQL_URL");
            let engine = connect_mysql_url(&url).await?;
            run_with_engine(&engine, db, scenario).await
        }
        DbKind::Postgres => {
            let url = db.url().expect("LDB_PG_URL");
            let engine = connect_pg_url(&url).await?;
            run_with_engine(&engine, db, scenario).await
        }
    }
}

async fn run_with_engine<E: Engine>(
    engine: &E,
    db: DbKind,
    scenario: Scenario,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match scenario {
        Scenario::FilterPage => {
            let _ = list::<BenchUser, _>(engine)
                .where_(filter_where())
                .order_by("id", Order::Asc)
                .limit(PAGE.limit)
                .offset(PAGE.offset)
                .await?;
        }
        Scenario::PageWithTotal => {
            let _ = count::<BenchUser, _>(engine).where_(filter_where()).await?;
            let _ = list::<BenchUser, _>(engine)
                .where_(filter_where())
                .order_by("id", Order::Asc)
                .limit(PAGE.limit)
                .offset(PAGE.offset)
                .await?;
        }
        Scenario::PartialUpdate => {
            let patch = BenchUser {
                id: None,
                name: None,
                age: Some(PATCH_AGE),
                status: None,
                city: Some(PATCH_CITY.to_string()),
                created_at: None,
            };
            update(engine, &patch)
                .where_(w().eq("status", 1i16))
                .await?;
        }
        Scenario::Upsert => {
            let mut user = BenchUser {
                id: None,
                name: Some(UPSERT_NAME.to_string()),
                age: Some(PATCH_AGE),
                status: Some(1),
                city: Some("shanghai".to_string()),
                created_at: None,
            };
            insert(engine, &mut user)
                .on_conflict(OnConflict::UpdateKey {
                    column_name_list: vec!["name".into()],
                })
                .await?;
        }
        Scenario::DeleteByIds => {
            let ids = delete_id_list();
            delete::<BenchUser, _>(engine)
                .where_(w().in_list("id", ids))
                .await?;
            setup::restore_deleted_ids(db).await?;
        }
        Scenario::GetOrInsert => {
            let mut candidate = BenchUser {
                id: None,
                name: Some(GET_OR_INSERT_NAME.to_string()),
                age: Some(20),
                status: Some(1),
                city: Some("shanghai".to_string()),
                created_at: None,
            };
            let _ = get_or_insert(engine, &mut candidate)
                .where_(w().eq("name", GET_OR_INSERT_NAME))
                .await?;
        }
    }
    Ok(())
}

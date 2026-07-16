//! Welds 后端。

use welds::prelude::*;

use crate::scenario::{
    FILTER, GET_OR_INSERT_NAME, PAGE, PATCH_AGE, PATCH_CITY, UPSERT_NAME, delete_id_list,
};
use crate::setup;
use crate::{DbKind, Scenario};

#[derive(Debug, Clone, WeldsModel)]
#[welds(table = "t_user")]
pub struct BenchUser {
    #[welds(primary_key)]
    pub id: i64,
    pub name: String,
    pub age: i32,
    pub status: i16,
    pub city: Option<String>,
}

async fn client(
    db: DbKind,
) -> Result<welds::connections::any::AnyClient, Box<dyn std::error::Error + Send + Sync>> {
    let url = db.url().expect("database url");
    Ok(welds::connections::connect(url).await?)
}

pub async fn run(
    db: DbKind,
    scenario: Scenario,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = client(db).await?;
    let client = client.as_ref();
    match scenario {
        Scenario::FilterPage => {
            let mut q = BenchUser::all();
            if let Some(pattern) = FILTER.name_like {
                q = q.where_col(|u| u.name.like(pattern));
            }
            if let Some(v) = FILTER.age_min {
                q = q.where_col(|u| u.age.gte(v));
            }
            if let Some(v) = FILTER.age_max {
                q = q.where_col(|u| u.age.lte(v));
            }
            if let Some(v) = FILTER.status {
                q = q.where_col(|u| u.status.equal(v));
            }
            if let Some(v) = FILTER.city {
                q = q.where_col(|u| u.city.equal(Some(v.to_string())));
            }
            let _ = q
                .order_by_asc(|u| u.id)
                .limit(PAGE.limit as i64)
                .offset(PAGE.offset as i64)
                .run(client)
                .await?;
        }
        Scenario::PageWithTotal => {
            let mut q = BenchUser::all();
            if let Some(pattern) = FILTER.name_like {
                q = q.where_col(|u| u.name.like(pattern));
            }
            if let Some(v) = FILTER.age_min {
                q = q.where_col(|u| u.age.gte(v));
            }
            if let Some(v) = FILTER.age_max {
                q = q.where_col(|u| u.age.lte(v));
            }
            if let Some(v) = FILTER.status {
                q = q.where_col(|u| u.status.equal(v));
            }
            if let Some(v) = FILTER.city {
                q = q.where_col(|u| u.city.equal(Some(v.to_string())));
            }
            let _ = q.count(client).await?;

            let mut q = BenchUser::all();
            if let Some(pattern) = FILTER.name_like {
                q = q.where_col(|u| u.name.like(pattern));
            }
            if let Some(v) = FILTER.age_min {
                q = q.where_col(|u| u.age.gte(v));
            }
            if let Some(v) = FILTER.age_max {
                q = q.where_col(|u| u.age.lte(v));
            }
            if let Some(v) = FILTER.status {
                q = q.where_col(|u| u.status.equal(v));
            }
            if let Some(v) = FILTER.city {
                q = q.where_col(|u| u.city.equal(Some(v.to_string())));
            }
            let _ = q
                .order_by_asc(|u| u.id)
                .limit(PAGE.limit as i64)
                .offset(PAGE.offset as i64)
                .run(client)
                .await?;
        }
        Scenario::PartialUpdate => {
            BenchUser::where_col(|u| u.status.equal(1i16))
                .set(|u| u.age, PATCH_AGE)
                .set(|u| u.city, Some(PATCH_CITY.to_string()))
                .run(client)
                .await?;
        }
        Scenario::Upsert => {
            // Welds 无统一 upsert API：先查后写，冲突则更新。
            let existing = BenchUser::where_col(|u| u.name.equal(UPSERT_NAME))
                .limit(1)
                .run(client)
                .await?;
            if let Some(mut row) = existing.into_iter().next() {
                row.age = PATCH_AGE;
                row.save(client).await?;
            } else {
                let mut user = BenchUser::new();
                user.name = UPSERT_NAME.to_string();
                user.age = PATCH_AGE;
                user.status = 1;
                user.city = Some("shanghai".to_string());
                user.save(client).await?;
            }
        }
        Scenario::DeleteByIds => {
            let ids = delete_id_list();
            BenchUser::where_col(|u| u.id.in_list(ids.to_vec().as_slice()))
                .delete(client)
                .await?;
            setup::restore_deleted_ids(db).await?;
        }
        Scenario::GetOrInsert => {
            let existing = BenchUser::where_col(|u| u.name.equal(GET_OR_INSERT_NAME))
                .limit(1)
                .run(client)
                .await?;
            if existing.is_empty() {
                let mut user = BenchUser::new();
                user.name = GET_OR_INSERT_NAME.to_string();
                user.age = 20;
                user.status = 1;
                user.city = Some("shanghai".to_string());
                user.save(client).await?;
            }
        }
    }
    Ok(())
}

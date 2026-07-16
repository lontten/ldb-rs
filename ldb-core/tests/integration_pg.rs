//! PostgreSQL 集成测试（需 `LDB_PG_URL` 环境变量；默认 `#[ignore]`，CI 用 `--include-ignored`）。

#![cfg(feature = "integration")]

use ldb_core::integration_support::{require_pg, sample_user};
use ldb_core::{
    Engine, OnConflict, Transaction, count, delete, first, get_or_insert, has, insert, list,
    test_util::TestUser, update, w,
};

#[tokio::test]
#[ignore = "requires LDB_PG_URL"]
async fn pg_crud_smoke() {
    let engine = require_pg().await;

    let mut user = sample_user("carol", 22);
    insert(&engine, &mut user).await.unwrap();

    let n = count::<TestUser, _>(&engine)
        .where_(w().eq("name", "carol"))
        .await
        .unwrap();
    assert_eq!(n, 1);

    let patch = ldb_core::test_util::TestUserWhere {
        name: None,
        age: Some(23),
    };
    update(&engine, &patch)
        .where_(w().eq("name", "carol"))
        .await
        .unwrap();

    delete::<TestUser, _>(&engine)
        .where_(w().eq("name", "carol"))
        .await
        .unwrap();

    assert!(
        !has::<TestUser, _>(&engine)
            .where_(w().gt("id", 0))
            .await
            .unwrap()
    );
}

#[tokio::test]
#[ignore = "requires LDB_PG_URL"]
async fn pg_select_first_list() {
    let engine = require_pg().await;

    let mut u1 = sample_user("pg_sel_a", 11);
    let mut u2 = sample_user("pg_sel_b", 22);
    insert(&engine, &mut u1).await.unwrap();
    insert(&engine, &mut u2).await.unwrap();

    let first_user = first::<TestUser, _>(&engine)
        .where_(w().eq("name", "pg_sel_a"))
        .await
        .unwrap()
        .expect("expected one row");
    assert_eq!(first_user.name.as_deref(), Some("pg_sel_a"));

    let rows = list::<TestUser, _>(&engine)
        .where_(w().gt("id", 0))
        .order_by("name", ldb_core::Order::Asc)
        .await
        .unwrap();
    assert_eq!(rows.len(), 2);
}

#[tokio::test]
#[ignore = "requires LDB_PG_URL"]
async fn pg_get_or_insert() {
    let engine = require_pg().await;

    let mut candidate = sample_user("pg_goi", 77);
    get_or_insert(&engine, &mut candidate)
        .where_(w().eq("name", "pg_goi"))
        .await
        .unwrap();

    let mut again = sample_user("pg_goi", 77);
    get_or_insert(&engine, &mut again)
        .where_(w().eq("name", "pg_goi"))
        .await
        .unwrap();

    assert_eq!(
        count::<TestUser, _>(&engine)
            .where_(w().eq("name", "pg_goi"))
            .await
            .unwrap(),
        1
    );
}

#[tokio::test]
#[ignore = "requires LDB_PG_URL"]
async fn pg_upsert_on_conflict() {
    let engine = require_pg().await;

    let mut user = sample_user("dave", 33);
    insert(&engine, &mut user)
        .on_conflict(OnConflict::DoNothing)
        .await
        .unwrap();
    insert(&engine, &mut user)
        .on_conflict(OnConflict::DoNothing)
        .await
        .unwrap();

    let n = count::<TestUser, _>(&engine)
        .where_(w().eq("name", "dave"))
        .await
        .unwrap();
    assert_eq!(n, 1);
}

#[tokio::test]
#[ignore = "requires LDB_PG_URL"]
async fn pg_insert_fillback_and_upsert_update() {
    let engine = require_pg().await;
    let mut user = sample_user("pg_upsert", 30);
    insert(&engine, &mut user).await.unwrap();
    assert!(user.id.is_some());

    let mut changed = sample_user("pg_upsert", 31);
    insert(&engine, &mut changed)
        .on_conflict(OnConflict::UpdateKey {
            column_name_list: vec!["name".into()],
        })
        .await
        .unwrap();
    let row = first::<TestUser, _>(&engine)
        .where_(w().eq("name", "pg_upsert"))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(row.age, Some(31));
}

#[tokio::test]
#[ignore = "requires LDB_PG_URL"]
async fn pg_transaction_commit() {
    let engine = require_pg().await;

    let tx: Transaction = engine.begin().await.unwrap();
    let mut user = sample_user("pg_tx", 44);
    insert(&tx, &mut user).await.unwrap();
    tx.commit().await.unwrap();

    assert!(
        has::<TestUser, _>(&engine)
            .where_(w().eq("name", "pg_tx"))
            .await
            .unwrap()
    );
}

#[tokio::test]
#[ignore = "requires LDB_PG_URL"]
async fn pg_transaction_rollback() {
    let engine = require_pg().await;

    let tx: Transaction = engine.begin().await.unwrap();
    let mut user = sample_user("pg_rollback", 55);
    insert(&tx, &mut user).await.unwrap();
    tx.rollback().await.unwrap();

    assert!(
        !has::<TestUser, _>(&engine)
            .where_(w().eq("name", "pg_rollback"))
            .await
            .unwrap()
    );
}

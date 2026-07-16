//! MySQL 集成测试（需 `LDB_MYSQL_URL` 环境变量；默认 `#[ignore]`，CI 用 `--include-ignored`）。

#![cfg(feature = "integration")]

use ldb_core::integration_support::{require_mysql, sample_user};
use ldb_core::{
    Engine, OnConflict, Transaction, count, delete, first, get_or_insert, has, insert, list,
    test_util::TestUser, update, w,
};

#[tokio::test]
#[ignore = "requires LDB_MYSQL_URL"]
async fn mysql_crud_smoke() {
    let engine = require_mysql().await;

    let mut user = sample_user("alice", 20);
    insert(&engine, &mut user).await.unwrap();

    let n = count::<TestUser, _>(&engine)
        .where_(w().eq("name", "alice"))
        .await
        .unwrap();
    assert_eq!(n, 1);

    assert!(
        has::<TestUser, _>(&engine)
            .where_(w().eq("name", "alice"))
            .await
            .unwrap()
    );

    let patch = ldb_core::test_util::TestUserWhere {
        name: None,
        age: Some(21),
    };
    update(&engine, &patch)
        .where_(w().eq("name", "alice"))
        .await
        .unwrap();

    delete::<TestUser, _>(&engine)
        .where_(w().eq("name", "alice"))
        .await
        .unwrap();

    let n = count::<TestUser, _>(&engine)
        .where_(w().gt("id", 0))
        .await
        .unwrap();
    assert_eq!(n, 0);
}

#[tokio::test]
#[ignore = "requires LDB_MYSQL_URL"]
async fn mysql_select_first_list() {
    let engine = require_mysql().await;

    let mut u1 = sample_user("sel_a", 10);
    let mut u2 = sample_user("sel_b", 20);
    insert(&engine, &mut u1).await.unwrap();
    insert(&engine, &mut u2).await.unwrap();

    let first_user = first::<TestUser, _>(&engine)
        .where_(w().eq("name", "sel_a"))
        .await
        .unwrap()
        .expect("expected one row");
    assert_eq!(first_user.name.as_deref(), Some("sel_a"));
    assert_eq!(first_user.age, Some(10));

    let rows = list::<TestUser, _>(&engine)
        .where_(w().gt("id", 0))
        .order_by("name", ldb_core::Order::Asc)
        .await
        .unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].name.as_deref(), Some("sel_a"));
    assert_eq!(rows[1].name.as_deref(), Some("sel_b"));
}

#[tokio::test]
#[ignore = "requires LDB_MYSQL_URL"]
async fn mysql_get_or_insert() {
    let engine = require_mysql().await;

    let mut candidate = sample_user("goi_user", 99);
    let inserted = get_or_insert(&engine, &mut candidate)
        .where_(w().eq("name", "goi_user"))
        .await
        .unwrap();
    assert_eq!(inserted.name.as_deref(), Some("goi_user"));

    let mut again = sample_user("goi_user", 99);
    let existing = get_or_insert(&engine, &mut again)
        .where_(w().eq("name", "goi_user"))
        .await
        .unwrap();
    assert_eq!(existing.name.as_deref(), Some("goi_user"));
    assert_eq!(
        count::<TestUser, _>(&engine)
            .where_(w().eq("name", "goi_user"))
            .await
            .unwrap(),
        1
    );
}

#[tokio::test]
#[ignore = "requires LDB_MYSQL_URL"]
async fn mysql_upsert_smoke() {
    let engine = require_mysql().await;

    let mut user = sample_user("bob", 30);
    insert(&engine, &mut user)
        .on_conflict(OnConflict::DoNothing)
        .await
        .unwrap();
    insert(&engine, &mut user)
        .on_conflict(OnConflict::DoNothing)
        .await
        .unwrap();

    let n = count::<TestUser, _>(&engine)
        .where_(w().eq("name", "bob"))
        .await
        .unwrap();
    assert_eq!(n, 1);
}

#[tokio::test]
#[ignore = "requires LDB_MYSQL_URL"]
async fn mysql_insert_fillback_and_upsert_update() {
    let engine = require_mysql().await;
    let mut user = sample_user("mysql_upsert", 30);
    insert(&engine, &mut user).await.unwrap();
    assert!(user.id.is_some());

    let mut changed = sample_user("mysql_upsert", 31);
    insert(&engine, &mut changed)
        .on_conflict(OnConflict::UpdateKey {
            column_name_list: vec!["name".into()],
        })
        .await
        .unwrap();
    assert_eq!(changed.id, user.id);
    let row = first::<TestUser, _>(&engine)
        .where_(w().eq("name", "mysql_upsert"))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(row.age, Some(31));
}

#[tokio::test]
#[ignore = "requires LDB_MYSQL_URL"]
async fn mysql_transaction_commit() {
    let engine = require_mysql().await;

    let tx: Transaction = engine.begin().await.unwrap();
    let mut user = sample_user("tx_user", 40);
    insert(&tx, &mut user).await.unwrap();
    tx.commit().await.unwrap();

    assert!(
        has::<TestUser, _>(&engine)
            .where_(w().eq("name", "tx_user"))
            .await
            .unwrap()
    );
}

#[tokio::test]
#[ignore = "requires LDB_MYSQL_URL"]
async fn mysql_transaction_rollback() {
    let engine = require_mysql().await;

    let tx: Transaction = engine.begin().await.unwrap();
    let mut user = sample_user("rollback_user", 50);
    insert(&tx, &mut user).await.unwrap();
    tx.rollback().await.unwrap();

    assert!(
        !has::<TestUser, _>(&engine)
            .where_(w().eq("name", "rollback_user"))
            .await
            .unwrap()
    );
}

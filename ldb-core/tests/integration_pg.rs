//! PostgreSQL 集成测试（需 `LDB_PG_URL` 环境变量）。

#![cfg(feature = "integration")]

use ldb_core::integration_support::{sample_user, setup_pg};
use ldb_core::{
    Engine, OnConflict, Transaction, count, delete, has, insert, test_util::TestUser, update, w,
};

#[tokio::test]
async fn pg_crud_smoke() {
    let Some(engine) = setup_pg().await else {
        eprintln!("skip pg_crud_smoke: LDB_PG_URL not set or DB unreachable");
        return;
    };

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
async fn pg_upsert_on_conflict() {
    let Some(engine) = setup_pg().await else {
        eprintln!("skip pg_upsert_on_conflict: LDB_PG_URL not set or DB unreachable");
        return;
    };

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
async fn pg_transaction_commit() {
    let Some(engine) = setup_pg().await else {
        eprintln!("skip pg_transaction_commit: LDB_PG_URL not set or DB unreachable");
        return;
    };

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

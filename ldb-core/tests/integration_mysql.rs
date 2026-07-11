//! MySQL 集成测试（需 `LDB_MYSQL_URL` 环境变量）。

#![cfg(feature = "integration")]

use ldb_core::integration_support::{sample_user, setup_mysql};
use ldb_core::{
    Engine, OnConflict, Transaction, count, delete, has, insert, test_util::TestUser, update, w,
};

#[tokio::test]
async fn mysql_crud_smoke() {
    let Some(engine) = setup_mysql().await else {
        eprintln!("skip mysql_crud_smoke: LDB_MYSQL_URL not set or DB unreachable");
        return;
    };

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
async fn mysql_upsert_smoke() {
    let Some(engine) = setup_mysql().await else {
        eprintln!("skip mysql_upsert_smoke: LDB_MYSQL_URL not set or DB unreachable");
        return;
    };

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
async fn mysql_transaction_commit() {
    let Some(engine) = setup_mysql().await else {
        eprintln!("skip mysql_transaction_commit: LDB_MYSQL_URL not set or DB unreachable");
        return;
    };

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
async fn mysql_transaction_rollback() {
    let Some(engine) = setup_mysql().await else {
        eprintln!("skip mysql_transaction_rollback: LDB_MYSQL_URL not set or DB unreachable");
        return;
    };

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

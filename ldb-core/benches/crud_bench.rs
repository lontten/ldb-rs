//! CRUD 性能基准（需 `integration` feature 与 `LDB_*_URL`）。

#![cfg(feature = "integration")]

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use ldb_core::{connect_mysql_url, connect_pg_url, insert, list, test_util::TestUser, w};

async fn bench_insert_mysql(n: usize) {
    let url = std::env::var("LDB_MYSQL_URL").expect("LDB_MYSQL_URL");
    let engine = connect_mysql_url(&url).await.unwrap();
    engine
        .exec_sql(
            "CREATE TABLE IF NOT EXISTS t_user (
            id BIGINT AUTO_INCREMENT PRIMARY KEY,
            name VARCHAR(255),
            age INT
        )",
        )
        .await
        .unwrap();
    engine.exec_sql("TRUNCATE TABLE t_user").await.unwrap();
    for i in 0..n {
        let mut user = TestUser {
            id: None,
            name: Some(format!("user_{i}")),
            age: Some(i as i32),
        };
        insert(&engine, &mut user).await.unwrap();
    }
}

async fn bench_list_mysql(n: usize) {
    let url = std::env::var("LDB_MYSQL_URL").expect("LDB_MYSQL_URL");
    let engine = connect_mysql_url(&url).await.unwrap();
    let _ = list::<TestUser, _>(&engine)
        .where_(w().gt("id", 0))
        .limit(n as u64)
        .await
        .unwrap();
}

fn mysql_benches(c: &mut Criterion) {
    if std::env::var("LDB_MYSQL_URL").is_err() {
        return;
    }
    let mut group = c.benchmark_group("mysql");
    group.sample_size(10);
    for n in [10usize, 50] {
        group.bench_with_input(BenchmarkId::new("insert", n), &n, |b, &n| {
            b.to_async(tokio::runtime::Runtime::new().unwrap())
                .iter(|| bench_insert_mysql(n));
        });
        group.bench_with_input(BenchmarkId::new("list", n), &n, |b, &n| {
            b.to_async(tokio::runtime::Runtime::new().unwrap())
                .iter(|| bench_list_mysql(n));
        });
    }
    group.finish();
}

async fn bench_insert_pg(n: usize) {
    let url = std::env::var("LDB_PG_URL").expect("LDB_PG_URL");
    let engine = connect_pg_url(&url).await.unwrap();
    engine
        .exec_sql(
            "CREATE TABLE IF NOT EXISTS t_user (
            id BIGSERIAL PRIMARY KEY,
            name VARCHAR(255),
            age INT
        )",
        )
        .await
        .unwrap();
    engine.exec_sql("TRUNCATE TABLE t_user").await.unwrap();
    for i in 0..n {
        let mut user = TestUser {
            id: None,
            name: Some(format!("user_{i}")),
            age: Some(i as i32),
        };
        insert(&engine, &mut user).await.unwrap();
    }
}

fn pg_benches(c: &mut Criterion) {
    if std::env::var("LDB_PG_URL").is_err() {
        return;
    }
    let mut group = c.benchmark_group("postgres");
    group.sample_size(10);
    for n in [10usize, 50] {
        group.bench_with_input(BenchmarkId::new("insert", n), &n, |b, &n| {
            b.to_async(tokio::runtime::Runtime::new().unwrap())
                .iter(|| bench_insert_pg(n));
        });
    }
    group.finish();
}

criterion_group!(benches, mysql_benches, pg_benches);
criterion_main!(benches);

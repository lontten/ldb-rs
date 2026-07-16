//! 真实业务场景性能基准（需 `LDB_MYSQL_URL` / `LDB_PG_URL`）。

use std::time::Duration;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use ldb_bench::{DbKind, OrmKind, Scenario, run_bench, setup};
use tokio::runtime::Runtime;

fn register_db(c: &mut Criterion, db: DbKind) {
    if !db.available() {
        return;
    }
    let rt = Runtime::new().expect("tokio runtime");
    rt.block_on(async {
        setup::prepare(db)
            .await
            .unwrap_or_else(|e| panic!("prepare {}: {e}", db.label()));
    });

    let db_label = db.label();
    for scenario in Scenario::ALL {
        let mut group = c.benchmark_group(format!("{db_label}/{}", scenario.label()));
        group.sample_size(10);
        group.warm_up_time(Duration::from_secs(1));
        group.measurement_time(Duration::from_secs(2));
        for orm in OrmKind::ALL {
            group.bench_function(BenchmarkId::from_parameter(orm.label()), |b| {
                b.to_async(&rt).iter(|| run_bench(db, orm, scenario));
            });
        }
        group.finish();
    }
}

fn scenario_compare(c: &mut Criterion) {
    register_db(c, DbKind::Mysql);
    register_db(c, DbKind::Postgres);
}

criterion_group!(benches, scenario_compare);
criterion_main!(benches);

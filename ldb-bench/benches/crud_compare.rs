//! 多 ORM CRUD 性能基准（需 `LDB_MYSQL_URL` / `LDB_PG_URL`）。

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use ldb_bench::{CrudOp, DbKind, OrmKind, run_bench};
use tokio::runtime::Runtime;

const SIZES: [usize; 2] = [10, 50];

fn register_db(c: &mut Criterion, db: DbKind) {
    if !db.available() {
        return;
    }
    let rt = Runtime::new().expect("tokio runtime");
    let db_label = db.label();
    for op in CrudOp::ALL {
        let mut group = c.benchmark_group(format!("{db_label}/{}", op.label()));
        group.sample_size(10);
        for n in SIZES {
            for orm in OrmKind::ALL {
                group.bench_with_input(BenchmarkId::new(orm.label(), n), &n, |b, &n| {
                    b.to_async(&rt).iter(|| run_bench(db, orm, op, n));
                });
            }
        }
        group.finish();
    }
}

fn crud_compare(c: &mut Criterion) {
    register_db(c, DbKind::Mysql);
    register_db(c, DbKind::Postgres);
}

criterion_group!(benches, crud_compare);
criterion_main!(benches);

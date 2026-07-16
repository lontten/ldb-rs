//! 多 ORM 真实业务场景基准公共类型与调度。

pub mod backends;
pub mod ddl;
pub mod filter_sql;
pub mod model;
pub mod scenario;
#[cfg(feature = "diesel")]
pub mod schema_diesel;
pub mod setup;

use std::future::Future;
use std::pin::Pin;

pub use scenario::{
    FILTER, GET_OR_INSERT_NAME, PAGE, PATCH_AGE, PATCH_CITY, SEED_N, UPSERT_NAME, delete_id_list,
    seed_row,
};

/// 数据库方言。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbKind {
    Mysql,
    Postgres,
}

impl DbKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Mysql => "mysql",
            Self::Postgres => "postgres",
        }
    }

    pub fn url(self) -> Option<String> {
        match self {
            Self::Mysql => std::env::var("LDB_MYSQL_URL").ok(),
            Self::Postgres => std::env::var("LDB_PG_URL").ok(),
        }
    }

    pub fn available(self) -> bool {
        self.url().is_some()
    }
}

/// 真实业务场景。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scenario {
    FilterPage,
    PageWithTotal,
    PartialUpdate,
    Upsert,
    DeleteByIds,
    GetOrInsert,
}

impl Scenario {
    pub const ALL: [Self; 6] = [
        Self::FilterPage,
        Self::PageWithTotal,
        Self::PartialUpdate,
        Self::Upsert,
        Self::DeleteByIds,
        Self::GetOrInsert,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::FilterPage => "filter_page",
            Self::PageWithTotal => "page_with_total",
            Self::PartialUpdate => "partial_update",
            Self::Upsert => "upsert",
            Self::DeleteByIds => "delete_by_ids",
            Self::GetOrInsert => "get_or_insert",
        }
    }
}

/// 参与对比的 ORM / 数据访问方案。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrmKind {
    Ldb,
    Sqlx,
    Seaorm,
    #[cfg(feature = "diesel")]
    Diesel,
    Welds,
    Rbatis,
    Ormlite,
}

impl OrmKind {
    #[cfg(feature = "diesel")]
    pub const ALL: [Self; 7] = [
        Self::Ldb,
        Self::Sqlx,
        Self::Seaorm,
        Self::Diesel,
        Self::Welds,
        Self::Rbatis,
        Self::Ormlite,
    ];

    #[cfg(not(feature = "diesel"))]
    pub const ALL: [Self; 6] = [
        Self::Ldb,
        Self::Sqlx,
        Self::Seaorm,
        Self::Welds,
        Self::Rbatis,
        Self::Ormlite,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Ldb => "ldb",
            Self::Sqlx => "sqlx",
            Self::Seaorm => "seaorm",
            #[cfg(feature = "diesel")]
            Self::Diesel => "diesel",
            Self::Welds => "welds",
            Self::Rbatis => "rbatis",
            Self::Ormlite => "ormlite",
        }
    }
}

/// 执行单次基准迭代（热路径不含建表/全量 seed）。
pub fn run_bench(
    db: DbKind,
    orm: OrmKind,
    scenario: Scenario,
) -> Pin<Box<dyn Future<Output = ()> + Send>> {
    Box::pin(async move {
        backends::run(db, orm, scenario).await.unwrap_or_else(|e| {
            panic!(
                "bench {}/{}/{}: {e}",
                db.label(),
                scenario.label(),
                orm.label()
            )
        });
    })
}

//! 多 ORM CRUD 基准公共类型与调度。

pub mod backends;
pub mod ddl;
pub mod schema_diesel;

use std::future::Future;
use std::pin::Pin;

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

/// CRUD 操作。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrudOp {
    Insert,
    Update,
    Delete,
    First,
    List,
    Count,
}

impl CrudOp {
    pub const ALL: [Self; 6] = [
        Self::Insert,
        Self::Update,
        Self::Delete,
        Self::First,
        Self::List,
        Self::Count,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Insert => "insert",
            Self::Update => "update",
            Self::Delete => "delete",
            Self::First => "first",
            Self::List => "list",
            Self::Count => "count",
        }
    }
}

/// 参与对比的 ORM / 数据访问方案。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrmKind {
    Ldb,
    Sqlx,
    Seaorm,
    Diesel,
    Welds,
    Rbatis,
    Ormlite,
}

impl OrmKind {
    pub const ALL: [Self; 7] = [
        Self::Ldb,
        Self::Sqlx,
        Self::Seaorm,
        Self::Diesel,
        Self::Welds,
        Self::Rbatis,
        Self::Ormlite,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Ldb => "ldb",
            Self::Sqlx => "sqlx",
            Self::Seaorm => "seaorm",
            Self::Diesel => "diesel",
            Self::Welds => "welds",
            Self::Rbatis => "rbatis",
            Self::Ormlite => "ormlite",
        }
    }
}

/// 执行单次基准迭代。
pub fn run_bench(
    db: DbKind,
    orm: OrmKind,
    op: CrudOp,
    n: usize,
) -> Pin<Box<dyn Future<Output = ()> + Send>> {
    Box::pin(async move {
        backends::run(db, orm, op, n)
            .await
            .unwrap_or_else(|e| panic!("bench {}/{}/{n}: {e}", db.label(), orm.label()));
    })
}

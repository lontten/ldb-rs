//! 各 ORM 后端实现。

#[cfg(feature = "diesel")]
mod diesel;
mod ldb;
mod ormlite;
mod rbatis;
mod seaorm;
mod sqlx_backend;
mod welds;

use crate::{DbKind, OrmKind, Scenario};

pub async fn run(
    db: DbKind,
    orm: OrmKind,
    scenario: Scenario,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match orm {
        OrmKind::Ldb => ldb::run(db, scenario).await,
        OrmKind::Sqlx => sqlx_backend::run(db, scenario).await,
        OrmKind::Seaorm => seaorm::run(db, scenario).await,
        #[cfg(feature = "diesel")]
        OrmKind::Diesel => diesel::run(db, scenario).await,
        OrmKind::Welds => welds::run(db, scenario).await,
        OrmKind::Rbatis => rbatis::run(db, scenario).await,
        OrmKind::Ormlite => ormlite::run(db, scenario).await,
    }
}

//! ldb ORM 核心：Engine、Dialect、条件与 CRUD Builder。

pub mod config;
pub mod connect;
pub mod crud;
pub mod dialect;
pub mod engine;
pub mod error;
pub mod exec;
pub mod model;
pub mod mysql_dialect;
pub mod on_conflict;
pub mod order;
pub mod pg_dialect;
pub mod sql_build;
pub mod sql_value;
pub mod where_builder;

#[cfg(feature = "integration")]
pub mod integration_support;

#[cfg(any(test, feature = "test-util", feature = "integration"))]
pub mod test_util;

pub use config::{MysqlConfig, MysqlVersion, PgConfig, PoolConfig};
pub use connect::{connect_mysql, connect_pg};
#[cfg(any(feature = "integration", test))]
pub use connect::{connect_mysql_url, connect_pg_url};
pub use crud::{
    CountBuilder, DeleteBuilder, FirstBuilder, GetOrInsertBuilder, HasBuilder, InsertBuilder,
    ListBuilder, UpdateBuilder, UpdateByPkBuilder, count, delete, first, get_or_insert, has,
    insert, list, update, update_by_primary_key,
};
pub use dialect::{Dialect, PlaceholderStyle};
pub use engine::{Engine, InsertResult, Transaction};
pub use error::LdbError;
pub use exec::MockExecutor;
pub use exec::{MysqlEngine, PgEngine};
pub use model::{ColumnMeta, LdbModel, TableConf};
pub use on_conflict::OnConflict;
pub use order::{Order, OrderBy};
pub use sql_value::{IntoSqlValue, SqlValue};
pub use where_builder::{WhereBuilder, w};

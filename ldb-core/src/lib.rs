//! ldb ORM 核心：Engine、Dialect、条件与扩展上下文。

pub mod config;
pub mod dialect;
pub mod engine;
pub mod error;
pub mod extra;
pub mod where_builder;

pub use config::{MysqlConfig, MysqlVersion, PgConfig, PoolConfig};
pub use dialect::{Dialect, PlaceholderStyle};
pub use engine::{Engine, Transaction};
pub use error::LdbError;
pub use extra::{ExtraContext, e};
pub use where_builder::{WhereBuilder, w};

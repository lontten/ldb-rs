//! CRUD 入口与 Builder。

mod common;
mod delete;
mod insert;
mod select;
mod update;

pub use delete::{DeleteBuilder, delete};
pub use insert::{InsertBuilder, insert};
pub use select::{
    CountBuilder, FirstBuilder, GetOrInsertBuilder, HasBuilder, ListBuilder, count, first,
    get_or_insert, has, has_or_insert, list,
};
pub use update::{UpdateBuilder, UpdateByPkBuilder, update, update_by_primary_key};

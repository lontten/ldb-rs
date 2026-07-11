//! 数据库执行引擎抽象。

use std::future::Future;

use crate::error::LdbError;
use crate::where_builder::WhereBuilder;

/// 事务句柄占位（骨架阶段；后续将持有 sqlx 事务）。
#[derive(Debug, Clone, Copy, Default)]
pub struct Transaction;

/// 可执行 SQL 的数据库引擎（连接或事务）。
pub trait Engine: Send + Sync {
    /// 检测连接是否可用。
    fn ping(&self) -> impl Future<Output = Result<(), LdbError>> + Send;

    /// 开启事务。
    fn begin(&self) -> impl Future<Output = Result<Transaction, LdbError>> + Send;

    /// 提交当前事务。
    fn commit(&self) -> impl Future<Output = Result<(), LdbError>> + Send;

    /// 回滚当前事务。
    fn rollback(&self) -> impl Future<Output = Result<(), LdbError>> + Send;

    /// 将 `WhereBuilder` 解析为 SQL 片段与绑定参数。
    fn to_where_sql(
        &self,
        where_builder: &WhereBuilder,
        primary_key_column_name_list: &[&str],
    ) -> Result<(String, Vec<String>), LdbError>;
}

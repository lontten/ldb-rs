//! ldb 统一错误类型。

use thiserror::Error;

/// ldb 操作失败时返回的错误。
#[derive(Debug, Error)]
pub enum LdbError {
    /// 连接不可用或已关闭
    #[error("连接不可用或已关闭")]
    NotConnected,

    /// 需要 WHERE 条件但未提供
    #[error("需要 WHERE 条件但未提供")]
    WhereRequired,

    /// 无 WHERE 的全表操作且未显式允许
    #[error("无 WHERE 的全表操作且未显式允许")]
    FullTableOpNotAllowed,

    /// 模型字段与列映射错误
    #[error("模型映射错误: {0}")]
    ModelMapping(String),

    /// SQL 生成或方言改写失败
    #[error("SQL 构建错误: {0}")]
    SqlBuild(String),

    /// sqlx 执行错误
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    /// 功能尚未实现（开发阶段）
    #[error("尚未实现")]
    NotImplemented,
}

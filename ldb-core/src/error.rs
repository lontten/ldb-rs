//! ldb 统一错误类型。

use thiserror::Error;

/// ldb 操作失败时返回的错误。
#[derive(Debug, Error)]
pub enum LdbError {
    /// 功能尚未实现（骨架阶段占位）。
    #[error("尚未实现")]
    NotImplemented,
}

//! 查询扩展上下文（排序、分页、表名覆盖等）。

/// 查询与写操作的扩展选项（骨架阶段为空壳）。
#[derive(Debug, Clone, Default)]
pub struct ExtraContext {}

/// 创建空的 `ExtraContext`。
pub fn e() -> ExtraContext {
    ExtraContext::default()
}

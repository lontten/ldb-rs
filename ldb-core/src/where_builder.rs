//! WHERE 条件构建器。

/// 链式 WHERE 条件构建器（骨架阶段为空壳）。
#[derive(Debug, Clone, Default)]
pub struct WhereBuilder {}

/// 创建空的 `WhereBuilder`。
pub fn w() -> WhereBuilder {
    WhereBuilder::default()
}

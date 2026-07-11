//! 查询排序方向。

/// 排序方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Order {
    Asc,
    Desc,
}

/// 单列排序。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderBy {
    pub column: String,
    pub order: Order,
}

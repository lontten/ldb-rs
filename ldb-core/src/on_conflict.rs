//! Insert 冲突处理策略。

/// 唯一键冲突时的行为。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OnConflict {
    /// 冲突时不做写入
    DoNothing,
    /// 按指定唯一键列执行 upsert 更新
    UpdateKey { column_name_list: Vec<String> },
    /// 冲突时更新所有非主键列
    UpdateAll,
}

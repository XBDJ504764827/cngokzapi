//! 计分（PROJECT §9.4）：本站自己的计分分两层。
//!
//! 1. **单条成绩的分数**：标准 GOKZ 公式（地图 tier + 名次 + TP/PRO 口径）。
//!    当前导入的成绩直接采用上游 `records.points`（该值即上游按同一公式算出的结果），
//!    未来实现本地公式后写入 `records.points_local`，并用上游数据做 oracle 对比。
//! 2. **玩家汇总**：同一 `(地图, 关卡)` 只取最佳一条，再按模式求和——本层已由本站实现，
//!    结果写入 `player_scores.points_local`。

/// 总分口径：同一 (地图, 关卡) 取最佳分后求和。
pub fn total_points(best_per_stage: impl IntoIterator<Item = i64>) -> i64 {
    best_per_stage.into_iter().sum()
}

/// 平均分：总分 / 计入关卡数，四舍五入；无成绩返回 0。
pub fn average_points(total: i64, stages: u32) -> i64 {
    if stages == 0 {
        0
    } else {
        (total as f64 / stages as f64).round() as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_is_sum_of_best() {
        assert_eq!(total_points([1000, 900, 800]), 2700);
        assert_eq!(total_points(std::iter::empty()), 0);
    }

    #[test]
    fn average_rounds_and_guards_zero() {
        assert_eq!(average_points(2700, 3), 900);
        assert_eq!(average_points(1000, 3), 333);
        assert_eq!(average_points(0, 0), 0);
    }
}

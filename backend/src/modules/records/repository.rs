//! 最新记录数据源占位：三模式各最新 20 条（日期倒序）。
//! 真实数据接入时改查 PG（按创建时间倒序），上层不动。

use super::models::RecordEntry;
use crate::modules::mock;

pub const PER_MODE: usize = 20;

fn name_pool() -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut pool = Vec::new();
    for mode in mock::MODES {
        for h in leaderboard_heads(mode) {
            if seen.insert(h) {
                pool.push(h.to_string());
            }
        }
    }
    for n in mock::NAME_POOL {
        if seen.insert(*n) {
            pool.push(n.to_string());
        }
    }
    pool
}

fn leaderboard_heads(mode: &str) -> Vec<&'static str> {
    match mode {
        "KZT" => vec!["ProGamer", "SpeedRunner", "TechMaster"],
        "SKZ" => vec!["JumpGod", "ProGamer", "BunnyhopKing"],
        _ => vec!["TechMaster", "KZ_Master", "ClimbFan"],
    }
}

fn mode_index(mode: &str) -> u64 {
    match mode {
        "KZT" => 0,
        "SKZ" => 1,
        _ => 2,
    }
}

pub fn mock_records() -> Vec<RecordEntry> {
    let pool = name_pool();
    let mut rows = Vec::with_capacity(PER_MODE * 3);
    for (mode_idx, mode) in mock::MODES.iter().enumerate() {
        let mut rng = mock::mulberry32(500 + mode_index(mode) * 31);
        for i in 0..PER_MODE {
            let days = (i / 2) as u64; // 每天约两条，按时间倒序即最新 20 条
            rows.push(RecordEntry {
                player: pool[(i + mode_idx * 7) % pool.len()].clone(),
                map: mock::MAPS[(rng() * mock::MAPS.len() as f64) as usize].to_string(),
                mode: mode.to_string(),
                time: mock::mock_record_time(&mut rng),
                points: 600 + (rng() * 351.0) as u32,
                date: mock::days_ago_to_date(days),
            });
        }
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn twenty_per_mode_desc() {
        let rows = mock_records();
        for mode in mock::MODES {
            let items: Vec<_> = rows.iter().filter(|r| &r.mode == mode).collect();
            assert_eq!(items.len(), PER_MODE);
            assert!(items.windows(2).all(|w| w[0].date >= w[1].date));
        }
    }
}

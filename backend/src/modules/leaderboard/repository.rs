//! 排行榜数据源占位：每模式 120 条确定性模拟数据。
//! 真实数据接入时改查 PG（排行 SQL）+ Redis（榜单缓存），上层 service/handlers 不动。

use super::models::LeaderboardEntry;
use crate::modules::mock;

pub const BOARD_SIZE: usize = 120;

struct Head {
    player: &'static str,
    maps_completed: u32,
    total_points: u64,
    avg_points: f32,
    records: u32,
}

fn heads(mode: &str) -> Vec<Head> {
    match mode {
        "KZT" => vec![
            Head {
                player: "ProGamer",
                maps_completed: 234,
                total_points: 45678,
                avg_points: 195.2,
                records: 567,
            },
            Head {
                player: "SpeedRunner",
                maps_completed: 221,
                total_points: 43210,
                avg_points: 195.5,
                records: 534,
            },
            Head {
                player: "TechMaster",
                maps_completed: 198,
                total_points: 39456,
                avg_points: 199.3,
                records: 489,
            },
        ],
        "SKZ" => vec![
            Head {
                player: "JumpGod",
                maps_completed: 198,
                total_points: 38456,
                avg_points: 194.2,
                records: 512,
            },
            Head {
                player: "ProGamer",
                maps_completed: 187,
                total_points: 35678,
                avg_points: 190.8,
                records: 478,
            },
            Head {
                player: "BunnyhopKing",
                maps_completed: 176,
                total_points: 33456,
                avg_points: 190.1,
                records: 423,
            },
        ],
        _ => vec![
            Head {
                player: "TechMaster",
                maps_completed: 176,
                total_points: 32456,
                avg_points: 184.4,
                records: 445,
            },
            Head {
                player: "KZ_Master",
                maps_completed: 165,
                total_points: 30123,
                avg_points: 182.6,
                records: 412,
            },
            Head {
                player: "ClimbFan",
                maps_completed: 154,
                total_points: 28789,
                avg_points: 186.9,
                records: 387,
            },
        ],
    }
}

fn mode_index(mode: &str) -> u64 {
    match mode {
        "KZT" => 0,
        "SKZ" => 1,
        _ => 2,
    }
}

/// 生成整榜（已按总分降序，rank 连续）。`mode` 仅取 KZT/SKZ/VNL，校验在 service 层。
pub fn mock_board(mode: &str) -> Vec<LeaderboardEntry> {
    let mut rng = mock::mulberry32(1000 + mode_index(mode) * 97);
    let heads = heads(mode);
    let mut rows: Vec<LeaderboardEntry> = heads
        .iter()
        .enumerate()
        .map(|(i, h)| {
            let days = (rng() * 10.0) as u32; // 头部默认近 10 天活跃
            LeaderboardEntry {
                rank: (i + 1) as u32,
                player: h.player.to_string(),
                steam_id64: mock_steam_id64(mode, i),
                avatar: mock::mock_avatar(&format!("{mode}:{}", h.player)),
                maps_completed: h.maps_completed,
                total_points: h.total_points,
                avg_points: h.avg_points,
                records: h.records,
                last_played: mock::days_ago_to_date(days as u64),
                last_played_days_ago: days,
            }
        })
        .collect();

    // 生成池剔除头部重名，保证 120 个唯一名称
    let pool: Vec<&str> = mock::NAME_POOL
        .iter()
        .copied()
        .filter(|n| !heads.iter().any(|h| h.player == *n))
        .collect();
    let mut points = heads.last().map(|h| h.total_points).unwrap_or(30000);
    for i in heads.len()..BOARD_SIZE {
        let idx = i - heads.len();
        let base = pool[idx % pool.len()];
        let name = if idx >= pool.len() {
            format!("{base}_{}", idx / pool.len() + 1)
        } else {
            base.to_string()
        };
        points = points.saturating_sub(80 + (rng() * 320.0) as u64).max(1000);
        let prev = &rows[i - 1];
        let maps = prev
            .maps_completed
            .saturating_sub((rng() * 4.0) as u32)
            .max(5);
        let records = prev.records.saturating_sub((rng() * 6.0) as u32).max(12);
        let days = (rng() * 61.0) as u32;
        rows.push(LeaderboardEntry {
            rank: (i + 1) as u32,
            player: name.clone(),
            steam_id64: mock_steam_id64(mode, i),
            avatar: mock::mock_avatar(&format!("{mode}:{name}")),
            maps_completed: maps,
            total_points: points,
            avg_points: ((150.0 + rng() * 48.0) * 10.0).round() as f32 / 10.0,
            records,
            last_played: mock::days_ago_to_date(days as u64),
            last_played_days_ago: days,
        });
    }
    rows
}

fn mock_steam_id64(mode: &str, i: usize) -> String {
    let num = 100_000_000u64 + mode_index(mode) * 20_000_000 + (i as u64) * 7919;
    format!("76561198{num:09}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn board_shape() {
        for mode in ["KZT", "SKZ", "VNL"] {
            let rows = mock_board(mode);
            assert_eq!(rows.len(), BOARD_SIZE);
            let names: HashSet<_> = rows.iter().map(|r| &r.player).collect();
            assert_eq!(names.len(), BOARD_SIZE, "{mode} 名称唯一");
            assert!(rows
                .windows(2)
                .all(|w| w[0].total_points >= w[1].total_points));
            assert!(rows.iter().all(|r| r.steam_id64.len() == 17));
        }
    }
}

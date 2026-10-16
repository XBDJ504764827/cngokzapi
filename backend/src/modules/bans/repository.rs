//! 封禁名单数据源占位：45 条（凑够 3 页验证翻页），按封禁时间倒序。
//! 真实数据接入时改查 PG（bans 表 + ILIKE 搜索 + COUNT），上层不动。

use super::models::BanEntry;
use crate::modules::mock;

pub const BANS_COUNT: usize = 45;

const REASONS: &[&str] = &[
    "使用连跳作弊",
    "穿墙作弊",
    "计时器数据篡改",
    "多开账号刷分",
    "恶意利用地图漏洞",
];

const SERVERS: &[&str] = &[
    "【CN】KZ 中国服务器 #1",
    "【CN】KZ 练习服 #2",
    "【CN】KZ 竞技服 #3",
];

pub fn mock_bans() -> Vec<BanEntry> {
    let mut rng = mock::mulberry32(777);
    // 名称池：头部熟人 + 通用池去重，不够 45 时加 _bN 后缀
    let mut seen = std::collections::HashSet::new();
    let mut pool: Vec<String> = Vec::new();
    for name in [
        "ProGamer",
        "SpeedRunner",
        "TechMaster",
        "JumpGod",
        "BunnyhopKing",
        "KZ_Master",
        "ClimbFan",
    ] {
        if seen.insert(name) {
            pool.push(name.to_string());
        }
    }
    for n in mock::NAME_POOL {
        if seen.insert(*n) {
            pool.push(n.to_string());
        }
    }
    let mut rows: Vec<(u32, BanEntry)> = Vec::with_capacity(BANS_COUNT);
    for i in 0..BANS_COUNT {
        let base = &pool[i % pool.len()];
        let name = if i >= pool.len() {
            format!("{base}_b{}", i / pool.len() + 1)
        } else {
            base.clone()
        };
        let map = mock::MAPS[(rng() * mock::MAPS.len() as f64) as usize];
        let mode = mock::MODES[(rng() * 3.0) as usize];
        let days = (rng() * 90.0) as u32;
        let num = 300_000_000u64 + (i as u64) * 104_729;
        rows.push((
            days,
            BanEntry {
                player: name.clone(),
                steam_id64: format!("76561198{num:09}"),
                avatar: mock::mock_avatar(&format!("ban:{name}")),
                reason: REASONS[(rng() * REASONS.len() as f64) as usize].to_string(),
                banned_at: mock::days_ago_to_date(days as u64),
                server: SERVERS[(rng() * SERVERS.len() as f64) as usize].to_string(),
                detail: format!(
                    "{map} · {mode} · {} · Demo #{}",
                    mock::mock_record_time(&mut rng),
                    80000 + (rng() * 19999.0) as u32
                ),
            },
        ));
    }
    rows.sort_by_key(|(days, _)| *days); // 最近被封禁的在前
    rows.into_iter().map(|(_, b)| b).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn shape_desc_unique() {
        let rows = mock_bans();
        assert_eq!(rows.len(), BANS_COUNT);
        let names: HashSet<_> = rows.iter().map(|b| &b.player).collect();
        assert_eq!(names.len(), BANS_COUNT);
        assert!(rows.iter().all(|b| b.steam_id64.len() == 17));
    }
}

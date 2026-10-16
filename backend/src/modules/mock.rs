//! 模拟数据公共件：占位阶段各 `repository.rs` 的确定性数据来源。
//! 真实数据接入（PG/Redis）时各 repository 改查库，本文件整体删除。

use std::time::{SystemTime, UNIX_EPOCH};

pub const MODES: &[&str] = &["KZT", "SKZ", "VNL"];

pub const MAPS: &[&str] = &[
    "kz_hopstep",
    "kz_easy_beta",
    "kz_extreme_tower",
    "kz_bhop_easy",
    "kz_surf_complex",
    "kz_climb_hard",
];

pub const NAME_POOL: &[&str] = &[
    "JumpGod",
    "BunnyhopKing",
    "StrafePro",
    "KZ_Master",
    "ClimbFan",
    "NewbieMaster",
    "AirControl",
    "StrafeKing",
    "BhopGod",
    "KZLegend",
    "SurfPro",
    "ClimbExpert",
    "TriggerHappy",
    "EdgeBug",
    "LongJump",
    "Bhopper",
    "SurfMaster",
    "KreedzFan",
    "TimerHunter",
    "Checkpoint",
    "NoScope",
    "PreStrafe",
    "SlideMaster",
    "CrouchBug",
    "LadderKing",
    "RampSlayer",
    "BlockMaster",
    "GapJumper",
    "TechNinja",
    "W Strafe",
];

/// 与旧原型前端一致的确定性伪随机（mulberry32 移植），保证每次重启榜单一致。
pub fn mulberry32(seed: u64) -> impl FnMut() -> f64 {
    let mut a = seed as u32;
    move || {
        a = a.wrapping_add(0x6D2B79F5);
        let mut t = a;
        t = (t ^ (t >> 15)).wrapping_mul(1 | t);
        t = t.wrapping_add((t ^ (t >> 7)).wrapping_mul(61 | t)) ^ t;
        ((t ^ (t >> 14)) as f64) / 4294967296.0
    }
}

/// 天数偏移 → `YYYY-MM-DD`。
pub fn days_ago_to_date(days_ago: u64) -> String {
    let today = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        / 86400;
    let (y, m, d) = civil_from_days(today as i64 - days_ago as i64);
    format!("{y:04}-{m:02}-{d:02}")
}

/// 天数 → 民用日期（Howard Hinnant 算法）。
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// 模拟头像：与旧原型同源（dicebear identicon），后续换 Steam 真实头像 URL。
pub fn mock_avatar(seed: &str) -> String {
    format!("https://api.dicebear.com/7.x/identicon/svg?seed={seed}")
}

/// 模拟成绩时间 `MM:SS.cc`。
pub fn mock_record_time(rng: &mut impl FnMut() -> f64) -> String {
    let m = (rng() * 4.0) as u32;
    let s = (rng() * 60.0) as u32;
    let ms = (rng() * 100.0) as u32;
    format!("{m:02}:{s:02}.{ms:02}")
}

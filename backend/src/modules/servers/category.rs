//! 社区分类（纯函数，无 IO）：把服务器归到所属社区，供列表按社区筛选。
//!
//! 规则：优先使用上游 `group_name`（如 `AXE GOKZ` → `AXE`）；缺失时从服务器名推导
//! （`【HIME】…` → `HIME`、`KZMANIA GOKZ 2` → `KZMANIA`、`House of Climb  EU …` → `House of Climb`）。
//! 归一化时去掉末尾的通用词与编号，使同一社区的不同叫法归为一类。

/// 归一化时从末尾去掉的通用词（大小写不敏感）。
const GENERIC_SUFFIX: &[&str] = &["GOKZ", "KZ", "Server", "社区", "攀岩竞速", "攀岩", "服"];

/// 分类：`group_name` 非空时优先；否则从 `name` 推导；都拿不到时归为「其他」。
pub fn community(name: &str, group_name: Option<&str>) -> String {
    let base = match group_name.map(str::trim).filter(|s| !s.is_empty()) {
        Some(g) => g.to_string(),
        None => derive_from_name(name),
    };
    let normalized = normalize(&base);
    if normalized.is_empty() {
        "其他".to_string()
    } else {
        normalized
    }
}

/// 从服务器名推导社区名：优先 `【】` / `[]` 前缀，其次取**最早出现**的分隔符前的前缀。
/// 不再按 marker 优先级命中（否则 `AXE GOKZ #B07 | WL` 会先被 ` |` 切开）。
fn derive_from_name(name: &str) -> String {
    if let Some(inner) = bracket_prefix(name) {
        return inner;
    }
    const MARKERS: &[&str] = &[
        "  ",
        " 攀岩",
        " |",
        "|",
        " GOKZ",
        " KZ",
        " Surf",
        " Advanced",
        " Tier",
        " #",
        " -",
        " 服",
        "服",
    ];
    let mut earliest: Option<usize> = None;
    for marker in MARKERS {
        if let Some(i) = name.find(marker) {
            if i > 0 && earliest.is_none_or(|b| i < b) {
                earliest = Some(i);
            }
        }
    }
    if let Some(i) = earliest {
        let prefix = name[..i].trim();
        if !prefix.is_empty() {
            return prefix.to_string();
        }
    }
    name.trim().to_string()
}

/// 名称开头的 `【X】`（中文括号）或 `[X]`（半角方括号）。
fn bracket_prefix(name: &str) -> Option<String> {
    let trimmed = name.trim_start();
    for (open, close) in [('【', '】'), ('[', ']')] {
        if let Some(rest) = trimmed.strip_prefix(open) {
            if let Some(end) = rest.find(close) {
                let inner = rest[..end].trim();
                if !inner.is_empty() {
                    return Some(inner.to_string());
                }
            }
        }
    }
    None
}

/// 去掉末尾的通用词与纯数字编号，并压缩空白。
fn normalize(s: &str) -> String {
    let mut tokens: Vec<&str> = s.split_whitespace().collect();
    while let Some(last) = tokens.last() {
        let generic = GENERIC_SUFFIX.iter().any(|g| last.eq_ignore_ascii_case(g));
        let numbered = last.chars().all(|c| c.is_ascii_digit());
        // 服务器编号 / 指令后缀：`#B07`、`#01`、`!ws`、`!knife`
        let server_code = last.starts_with('#') || last.starts_with('!');
        if generic || numbered || server_code {
            tokens.pop();
        } else {
            break;
        }
    }
    tokens.join(" ")
}

#[cfg(test)]
mod tests {
    use super::community;

    #[test]
    fn prefers_group_name_and_strips_generic_suffix() {
        assert_eq!(community("CNGOKZ 1服", Some("CNGOKZ")), "CNGOKZ");
        assert_eq!(community("AXE GOKZ #B01", Some("AXE GOKZ")), "AXE");
        assert_eq!(
            community("House of Climb  EU GOKZ 1", Some("House of Climb")),
            "House of Climb"
        );
    }

    #[test]
    fn derives_from_name_when_group_missing() {
        assert_eq!(community("CNGOKZ 2服", None), "CNGOKZ");
        assert_eq!(community("AXE GOKZ #B03", None), "AXE");
        assert_eq!(
            community("【HIME】GOKZ [T1-T3] 广州 在线[2/10]", None),
            "HIME"
        );
        assert_eq!(community("KZMANIA GOKZ 2", None), "KZMANIA");
        assert_eq!(
            community("UB社区 攀岩竞速 #01 GOKZ T1-T3 Q群 477871624", None),
            "UB社区"
        );
        assert_eq!(
            community("House of Climb  EU GOKZ 3", None),
            "House of Climb"
        );
        assert_eq!(community("Sunl 1服", None), "Sunl");
        assert_eq!(
            community("BadServers.net | GOKZ 1 [Tier 1-4] | Chicago", None),
            "BadServers.net"
        );
        assert_eq!(community("Celerity - Pre/Nopre KZ", None), "Celerity");
        assert_eq!(community("sneaK's GOKZ Climb !ws|!knife", None), "sneaK's");
        assert_eq!(community("AXE GOKZ #B07 | WL", None), "AXE");
        assert_eq!(community("AXE GOKZ #B08 | WL", None), "AXE");
        assert_eq!(
            community("sneaK's Advanced Surf Tier 3-6|!ws|!knife", None),
            "sneaK's"
        );
        assert_eq!(
            community("sneaK's KZ Climb !ws|!knife|!gloves", None),
            "sneaK's"
        );
    }

    #[test]
    fn falls_back_to_other() {
        assert_eq!(community("", None), "其他");
        assert_eq!(community("GOKZ", None), "其他");
    }
}

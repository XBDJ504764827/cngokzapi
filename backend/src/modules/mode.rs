//! 模式（KZT / SKZ / VNL）常量与校验：全站唯一口径，与前端 `types/common.ts` 一致。

use crate::errors::AppError;

/// 全站三个模式。
pub const MODES: [&str; 3] = ["KZT", "SKZ", "VNL"];

/// 校验并归一化模式参数；缺省 KZT，非法直接 400。
pub fn validate(mode: Option<&str>) -> Result<&str, AppError> {
    let mode = mode.unwrap_or("KZT");
    if !MODES.contains(&mode) {
        return Err(AppError::BadRequest(format!(
            "mode 非法，仅支持 {}",
            MODES.join("/")
        )));
    }
    Ok(mode)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_modes() {
        assert_eq!(validate(None).unwrap(), "KZT");
        assert_eq!(validate(Some("VNL")).unwrap(), "VNL");
        assert!(validate(Some("XXX")).is_err());
    }
}

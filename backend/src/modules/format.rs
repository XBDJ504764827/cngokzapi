//! 展示层格式化（纯函数，无 IO）。

/// 成绩耗时：毫秒整数 → GOKZ 习惯写法（`M:SS.mmm`，不足一分钟写 `SS.mmm`）。
pub fn format_duration_ms(ms: i32) -> String {
    let ms = ms.max(0);
    let minutes = ms / 60_000;
    let seconds = (ms % 60_000) / 1000;
    let millis = ms % 1000;
    if minutes > 0 {
        format!("{minutes}:{seconds:02}.{millis:03}")
    } else {
        format!("{seconds}.{millis:03}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_duration() {
        assert_eq!(format_duration_ms(0), "0.000");
        assert_eq!(format_duration_ms(9_876), "9.876");
        assert_eq!(format_duration_ms(69_422), "1:09.422");
        assert_eq!(format_duration_ms(3_600_000), "60:00.000");
        assert_eq!(format_duration_ms(-5), "0.000");
    }
}

//! 人类可读小数串 ↔ `i128` 缩放整数（对齐 `zepto-types::fixed`，ADR-0026）。
//!
//! - **API / WS 读路径**：金额为 decimal `String`（如 `"100000.000000"`）
//! - **Action 构造 / L1 msgpack**：内部仍为 `i128`；用 [`parse_decimal`] 从人类可读串转换
//! - **超精度拒绝**：小数位多于 `decimals` 时返回 `None`（不静默截断）

use serde::{Deserialize, Serialize};

/// 10^6 — 余额 / 保证金 / PnL / 手续费 / 费率精度。
pub const SCALE_6: i128 = 1_000_000;

/// 与 [`SCALE_6`] 配对的小数位数；市场精度上限 `MAX_DECIMALS`。
pub const DECIMALS_6: u32 = 6;

/// 把 `value`（整数）放大到目标精度（`value × 10^decimals`）。溢出 → `None`。
#[inline]
pub fn scale_int(value: i128, decimals: u32) -> Option<i128> {
    let mult = 10i128.checked_pow(decimals)?;
    value.checked_mul(mult)
}

/// 解析人类可读小数串为目标精度的 `i128`。
///
/// 小数位多于 `decimals` 一律拒绝（返回 `None`）。
pub fn parse_decimal(s: &str, decimals: u32) -> Option<i128> {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return None;
    }
    let (negative, rest) = match bytes[0] {
        b'+' => (false, &bytes[1..]),
        b'-' => (true, &bytes[1..]),
        _ => (false, bytes),
    };
    if rest.is_empty() {
        return None;
    }
    let dot = rest.iter().position(|&b| b == b'.');
    let (int_part, frac_part): (&[u8], &[u8]) = match dot {
        Some(idx) => {
            let frac = &rest[idx + 1..];
            if frac.contains(&b'.') {
                return None;
            }
            (&rest[..idx], frac)
        }
        None => (rest, b""),
    };
    if int_part.is_empty() && frac_part.is_empty() {
        return None;
    }
    let dec_usize = usize::try_from(decimals).ok()?;
    if frac_part.len() > dec_usize {
        return None;
    }
    let mut acc: i128 = 0;
    for &b in int_part {
        if !b.is_ascii_digit() {
            return None;
        }
        acc = acc.checked_mul(10)?.checked_add(i128::from(b - b'0'))?;
    }
    for &b in frac_part {
        if !b.is_ascii_digit() {
            return None;
        }
        acc = acc.checked_mul(10)?.checked_add(i128::from(b - b'0'))?;
    }
    let pad = dec_usize.checked_sub(frac_part.len())?;
    for _ in 0..pad {
        acc = acc.checked_mul(10)?;
    }
    if negative {
        acc = acc.checked_neg()?;
    }
    Some(acc)
}

/// 格式化 `i128` 缩放整数为人类可读小数串（小数部分补齐到 `decimals` 位）。
pub fn format_decimal(raw: i128, decimals: u32) -> String {
    if decimals == 0 {
        return raw.to_string();
    }
    let neg = raw < 0;
    let mag = raw.unsigned_abs();
    let scale = 10u128.pow(decimals);
    let int_part = mag / scale;
    let frac_part = mag % scale;
    let dec = decimals as usize;
    let sign = if neg { "-" } else { "" };
    format!("{sign}{int_part}.{frac_part:0width$}", width = dec)
}

/// Action wire 边界的 canonical 小数串（serde 透明为 `String`）。
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DecimalStr(pub String);

impl DecimalStr {
    #[inline]
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[inline]
    pub fn parse(&self, decimals: u32) -> Option<i128> {
        parse_decimal(&self.0, decimals)
    }

    #[inline]
    pub fn from_raw(raw: i128, decimals: u32) -> Self {
        Self(format_decimal(raw, decimals))
    }
}

impl From<&str> for DecimalStr {
    #[inline]
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for DecimalStr {
    #[inline]
    fn from(s: String) -> Self {
        Self(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_decimal_basic_and_strict() {
        assert_eq!(parse_decimal("1000.5", 6), Some(1_000_500_000));
        assert_eq!(parse_decimal("67231.5", 1), Some(672315));
        assert_eq!(parse_decimal("0.001", 5), Some(100));
        assert_eq!(parse_decimal("-1.234", 6), Some(-1_234_000));
    }

    #[test]
    fn parse_decimal_rejects_over_precision() {
        assert_eq!(parse_decimal("1.23456789", 6), None);
        assert_eq!(parse_decimal("67231.55", 1), None);
    }

    #[test]
    fn format_decimal_pads() {
        assert_eq!(format_decimal(672315, 1), "67231.5");
        assert_eq!(format_decimal(100, 5), "0.00100");
        assert_eq!(format_decimal(-1_234_000, 6), "-1.234000");
    }

    #[test]
    fn format_parse_round_trip() {
        for (raw, dec) in [
            (1_000_500_000i128, 6),
            (672315, 1),
            (100, 5),
            (-1_234_000, 6),
        ] {
            let s = format_decimal(raw, dec);
            assert_eq!(parse_decimal(&s, dec), Some(raw), "round trip {s}");
        }
    }
}

//! 年額を月ごとの額に直す（表示用の目安）

use super::params::*;

/// 住民税を給与から天引き（特別徴収）する場合の月額 (6月分, 7月〜翌5月分)
///
/// 年税額を12回に分け、100円未満の端数は最初の月（6月分）にまとめる。
pub fn resident_special_collection(annual: i64) -> (i64, i64) {
    let annual = annual.max(0);
    let rest = annual / RESIDENT_INSTALLMENTS / TAX_UNIT * TAX_UNIT;
    (annual - rest * (RESIDENT_INSTALLMENTS - 1), rest)
}

/// 年額の月平均（1円未満切捨て）
pub fn monthly_average(annual: i64) -> i64 {
    annual.div_euclid(MONTHS_PER_YEAR)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn special_collection() {
        // 240,500 ÷ 12 = 20,041.6… → 7月以降 20,000、6月 240,500 − 20,000×11 = 20,500
        assert_eq!(resident_special_collection(240_500), (20_500, 20_000));
        // 割り切れる場合は全月同額
        assert_eq!(resident_special_collection(240_000), (20_000, 20_000));
        // 100円未満の端数の境界：12×100 = 1,200円ごとに7月以降の額が上がる
        assert_eq!(resident_special_collection(241_100), (21_100, 20_000));
        assert_eq!(resident_special_collection(241_200), (20_100, 20_100));
        // 均等割だけ（5,000円）：5,000 ÷ 12 = 416 → 400、6月 600
        assert_eq!(resident_special_collection(5_000), (600, 400));
        assert_eq!(resident_special_collection(0), (0, 0));
        // 合計は年税額に一致する
        let (first, rest) = resident_special_collection(333_300);
        assert_eq!(first + rest * 11, 333_300);
    }

    #[test]
    fn average() {
        assert_eq!(monthly_average(3_908_800), 325_733);
        assert_eq!(monthly_average(0), 0);
    }
}

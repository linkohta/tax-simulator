//! 雑所得（公的年金等、副業）

/// 公的年金等に係る雑所得の速算表の1行：雑所得 = 収入 × 率 − 控除額（所法35条4項）
pub struct PensionRow {
    /// 収入がこの額未満の場合に適用
    pub below: i64,
    pub rate_pct: i64,
    pub subtract: i64,
}

/// 65歳未満（公的年金等に係る雑所得以外の合計所得金額が1,000万円以下の場合。
/// 国税庁タックスアンサー No.1600、令和2年分以後、令和8年4月1日現在）
pub static PENSION_NET_UNDER_65: &[PensionRow] = &[
    PensionRow { below: 1_300_000, rate_pct: 100, subtract: 600_000 },
    PensionRow { below: 4_100_000, rate_pct: 75, subtract: 275_000 },
    PensionRow { below: 7_700_000, rate_pct: 85, subtract: 685_000 },
    PensionRow { below: 10_000_000, rate_pct: 95, subtract: 1_455_000 },
    PensionRow { below: i64::MAX, rate_pct: 100, subtract: 1_955_000 },
];
/// 65歳以上（同上）
pub static PENSION_NET_65_AND_OVER: &[PensionRow] = &[
    PensionRow { below: 3_300_000, rate_pct: 100, subtract: 1_100_000 },
    PensionRow { below: 4_100_000, rate_pct: 75, subtract: 275_000 },
    PensionRow { below: 7_700_000, rate_pct: 85, subtract: 685_000 },
    PensionRow { below: 10_000_000, rate_pct: 95, subtract: 1_455_000 },
    PensionRow { below: i64::MAX, rate_pct: 100, subtract: 1_955_000 },
];
/// 公的年金等控除で「65歳以上」の表を使う年齢
pub const PENSION_ELDERLY_AGE_FROM: u32 = 65;
/// 公的年金等に係る雑所得以外の合計所得金額の区分。これを1つ超えるごとに控除額が減る
pub const PENSION_OTHER_INCOME_TIERS: [i64; 2] = [10_000_000, 20_000_000];
/// 上の区分を1つ超えるごとに減る控除額
pub const PENSION_DEDUCTION_REDUCTION_PER_TIER: i64 = 100_000;

/// 所得金額調整控除（給与所得と年金所得の双方を有する者）：
/// 給与所得と公的年金等に係る雑所得をそれぞれこの額まで合計し、この額を引く（措法41条の3の11第2項）
pub const SALARY_PENSION_ADJUSTMENT_CAP: i64 = 100_000;

/// 給与所得者は、給与・退職所得以外の所得がこの額以下なら所得税の確定申告が不要（所法121条1項）
pub const FILING_OTHER_INCOME_OVER: i64 = 200_000;
/// 公的年金等の収入がこの額以下で、ほかの所得が20万円以下なら所得税の確定申告が不要（所法121条3項）
pub const PENSION_FILING_EXEMPT_UNTIL: i64 = 4_000_000;

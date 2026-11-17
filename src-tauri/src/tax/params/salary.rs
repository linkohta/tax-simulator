//! 給与所得控除、所得金額調整控除

// ───────────── 給与所得控除 ─────────────

/// 給与所得控除の最低保障額（所得税：令和8・9年分は措法29条の4により74万円）
pub const SALARY_DEDUCTION_MIN_IT: i64 = 740_000;
/// 給与所得控除の最低保障額（住民税：令和9・10年度分は74万円）
pub const SALARY_DEDUCTION_MIN_RT: i64 = 740_000;
/// 給与所得の金額の特例（措法29条の4第2項の端数表）：(収入下限, 収入上限未満, 給与所得)
pub static SALARY_NET_SPECIAL: &[(i64, i64, i64)] = &[
    (2_191_000, 2_193_000, 1_451_000),
    (2_193_000, 2_196_000, 1_453_000),
    (2_196_000, 2_200_000, 1_456_000),
];

/// 所得金額調整控除（子ども・特別障害者等を有する者等）：給与収入がこの額を超えると対象
pub const INCOME_ADJUSTMENT_FROM: i64 = 8_500_000;
/// 同：計算に使う給与収入の上限
pub const INCOME_ADJUSTMENT_SALARY_CAP: i64 = 10_000_000;
/// 同：控除率（%）
pub const INCOME_ADJUSTMENT_RATE_PCT: i64 = 10;


// ───────────── 給与所得控除の算式 ─────────────

/// 給与所得控除額 = 収入 × 率 + 加算額（所法28条3項）
pub struct SalaryDeductionRow {
    /// この収入以下に適用
    pub upper: i64,
    pub rate_pct: i64,
    pub add: i64,
}
pub static SALARY_DEDUCTION_TABLE: &[SalaryDeductionRow] = &[
    SalaryDeductionRow { upper: 1_800_000, rate_pct: 40, add: -100_000 },
    SalaryDeductionRow { upper: 3_600_000, rate_pct: 30, add: 80_000 },
    SalaryDeductionRow { upper: 6_600_000, rate_pct: 20, add: 440_000 },
    SalaryDeductionRow { upper: 8_500_000, rate_pct: 10, add: 1_100_000 },
];
/// 給与所得控除額の上限（収入850万円超）
pub const SALARY_DEDUCTION_MAX: i64 = 1_950_000;
/// 収入がこの額未満なら所得税法別表第五による（収入を刻みの下端に切り下げて算式に当てはめる）
pub const SALARY_TABLE_UNTIL: i64 = 6_600_000;
/// 別表第五の刻み
pub const SALARY_TABLE_STEP: i64 = 4_000;


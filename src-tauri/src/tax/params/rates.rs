//! 税率（所得税の速算表、住民税率、株式の税率）と税額控除の率

// ───────────── 所得税 ─────────────

pub struct Bracket {
    pub upper: i64,
    pub rate_pct: i64,
    pub deduction: i64,
}

pub static INCOME_TAX_BRACKETS: &[Bracket] = &[
    Bracket { upper: 1_949_000, rate_pct: 5, deduction: 0 },
    Bracket { upper: 3_299_000, rate_pct: 10, deduction: 97_500 },
    Bracket { upper: 6_949_000, rate_pct: 20, deduction: 427_500 },
    Bracket { upper: 8_999_000, rate_pct: 23, deduction: 636_000 },
    Bracket { upper: 17_999_000, rate_pct: 33, deduction: 1_536_000 },
    Bracket { upper: 39_999_000, rate_pct: 40, deduction: 2_796_000 },
    Bracket { upper: i64::MAX, rate_pct: 45, deduction: 4_796_000 },
];

/// 復興特別所得税 2.1%（1/1000 %単位）
pub const RECONSTRUCTION_RATE_X1000: i64 = 2_100;

/// 上場株式等の源泉徴収税率：所得税・復興税 15.315%（1/1000 %単位）、住民税 5%
pub const STOCK_IT_RATE_X1000: i64 = 15_315;
pub const STOCK_RT_RATE_PCT: i64 = 5;

/// 配当控除で率が変わる課税総所得金額等の区切り（所法92条）
pub const DIVIDEND_CREDIT_THRESHOLD: i64 = 10_000_000;
/// 配当控除率（1/1000 %単位）。
/// 行は [剰余金の配当等, 証券投資信託, 外貨建等証券投資信託]、
/// 列は [課税総所得金額等1,000万円以下の部分, 超える部分]。
/// 所得税：所法92条・措法9条。住民税：地方税法附則5条
pub const DIVIDEND_CREDIT_IT: [[i64; 2]; 3] = [[10_000, 5_000], [5_000, 2_500], [2_500, 1_250]];
pub const DIVIDEND_CREDIT_RT_MUNI: [[i64; 2]; 3] = [[1_600, 800], [800, 400], [400, 200]];
pub const DIVIDEND_CREDIT_RT_PREF: [[i64; 2]; 3] = [[1_200, 600], [600, 300], [300, 150]];

/// 外国税額控除の住民税の控除限度額：所得税の控除限度額に対する率（%）。
/// 道府県民税12%（地方税法施行令7条の19）、市町村民税18%（同48条の9の2）。指定都市の6%・24%は扱わない
pub const FOREIGN_TAX_CREDIT_RT_PREF_PCT: i64 = 12;
pub const FOREIGN_TAX_CREDIT_RT_MUNI_PCT: i64 = 18;


// ───────────── 税率 ─────────────

/// 申告分離課税の上場株式等の所得税率（%。措法8条の4・37条の11）
pub const STOCK_SEPARATE_IT_RATE_PCT: i64 = 15;
/// 住民税所得割の標準税率（%）[市町村, 道府県]（地方税法314条の3・35条）
pub const RESIDENT_RATE_PCT: [i64; 2] = [6, 4];
/// 申告分離課税の上場株式等の住民税率（%）[市町村, 道府県]（地方税法附則33条の2・35条の2の2）
pub const STOCK_SEPARATE_RT_RATE_PCT: [i64; 2] = [3, 2];

/// 調整控除（地方税法37条・314条の6）：合計所得金額がこの額を超えると適用なし
pub const ADJUSTMENT_CREDIT_INCOME_LIMIT: i64 = 25_000_000;
/// 調整控除：合計課税所得金額がこの額以下かどうかで算式が変わる
pub const ADJUSTMENT_CREDIT_THRESHOLD: i64 = 2_000_000;
/// 調整控除の率（%）[市町村, 道府県]
pub const ADJUSTMENT_CREDIT_RATE_PCT: [i64; 2] = [3, 2];
/// 調整控除の最低額 [市町村, 道府県]（合計2,500円。200万円超の算式で下回る場合）
pub const ADJUSTMENT_CREDIT_MIN: [i64; 2] = [1_500, 1_000];


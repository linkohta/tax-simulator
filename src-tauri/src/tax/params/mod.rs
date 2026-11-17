//! 税制パラメータ（令和8年分所得税・令和9年度住民税／2026年度社会保険料）
//!
//! 金額は円。率は注記した単位の整数で持つ（浮動小数点の誤差を避けるため）。
//!
//! 分野ごとのファイルに分けてあり、ここですべて再公開する（`params::*` でどれも使える）。

mod deductions;
mod housing_loan;
mod misc;
mod personal;
mod rates;
mod resident;
mod salary;
mod social;

pub use deductions::*;
pub use housing_loan::*;
pub use misc::*;
pub use personal::*;
pub use rates::*;
pub use resident::*;
pub use salary::*;
pub use social::*;

// ───────────── 対象の年・単位・端数 ─────────────

/// 計算の対象とする年分（西暦）
pub const TAX_YEAR: u32 = 2026;
/// 1年の月数
pub const MONTHS_PER_YEAR: i64 = 12;

/// 年収別のグラフ：給与収入の刻み
pub const SALARY_CURVE_STEP: i64 = 500_000;
/// 同：横軸の上限を、今の給与収入の何%までにするか
pub const SALARY_CURVE_RANGE_PCT: i64 = 150;
/// 同：横軸の上限の最小値と最大値
pub const SALARY_CURVE_MIN_RANGE: i64 = 15_000_000;
pub const SALARY_CURVE_MAX_RANGE: i64 = 50_000_000;

/// 「1/1000 %単位」の率を金額に掛けるときの分母（9.85% = 9_850 / 100_000）
pub const RATE_X1000_DENOM: i64 = 100_000;
/// 課税所得金額の端数：1,000円未満切捨て（国税通則法118条、地方税法20条の4の2）
pub const TAXABLE_INCOME_UNIT: i64 = 1_000;
/// 税額の端数：100円未満切捨て（国税通則法119条、地方税法20条の4の2）
pub const TAX_UNIT: i64 = 100;

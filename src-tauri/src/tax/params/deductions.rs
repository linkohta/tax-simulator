//! 生命保険料控除・地震保険料控除・医療費控除、寄附金

// ───────────── 保険料控除 ─────────────

/// 令和8年分：23歳未満の扶養親族がいる場合の一般生命保険料（新契約）控除上限
pub const LIFE_GENERAL_CAP_SPECIAL_IT: i64 = 60_000;

// ───────────── 生命保険料控除・地震保険料控除・医療費控除 ─────────────

/// 生命保険料控除の算式の区切り（所法76条、地方税法34条1項5号）。
/// `full_until` 以下は全額、`half_until` 以下は 1/2 + 定額、`quarter_until` 以下は 1/4 + 定額、超えると上限。
/// 定額と上限は、区切りで金額が連続することから決まる（deductions::life_amount）。
pub struct LifeInsuranceTier {
    pub full_until: i64,
    pub half_until: i64,
    pub quarter_until: i64,
}
/// 所得税・新契約（上限4万円）
pub const LIFE_NEW_IT: LifeInsuranceTier =
    LifeInsuranceTier { full_until: 20_000, half_until: 40_000, quarter_until: 80_000 };
/// 所得税・一般生命保険料の新契約で23歳未満の扶養親族がいる場合（上限6万円。令和8年分の特例）
pub const LIFE_NEW_SPECIAL_IT: LifeInsuranceTier =
    LifeInsuranceTier { full_until: 30_000, half_until: 60_000, quarter_until: 120_000 };
/// 所得税・旧契約（上限5万円）
pub const LIFE_OLD_IT: LifeInsuranceTier =
    LifeInsuranceTier { full_until: 25_000, half_until: 50_000, quarter_until: 100_000 };
/// 住民税・新契約（上限2.8万円）
pub const LIFE_NEW_RT: LifeInsuranceTier =
    LifeInsuranceTier { full_until: 12_000, half_until: 32_000, quarter_until: 56_000 };
/// 住民税・旧契約（上限3.5万円）
pub const LIFE_OLD_RT: LifeInsuranceTier =
    LifeInsuranceTier { full_until: 15_000, half_until: 40_000, quarter_until: 70_000 };
/// 生命保険料控除の合計の上限（一般・介護医療・個人年金の合計）
pub const LIFE_TOTAL_CAP_IT: i64 = 120_000;
pub const LIFE_TOTAL_CAP_RT: i64 = 70_000;

/// 地震保険料控除（所法77条、地方税法34条1項5号の3）
pub struct EarthquakeTable {
    /// 地震保険料を割る数（所得税は全額＝1、住民税は1/2＝2）
    pub premium_div: i64,
    /// 地震保険料分の上限、かつ旧長期損害保険料との合計の上限
    pub cap: i64,
    /// 旧長期損害保険料：この額以下は全額
    pub old_full_until: i64,
    /// 旧長期損害保険料：この額以下は 1/2 + 定額、超えると上限
    pub old_half_until: i64,
}
pub const EARTHQUAKE_IT: EarthquakeTable =
    EarthquakeTable { premium_div: 1, cap: 50_000, old_full_until: 10_000, old_half_until: 20_000 };
pub const EARTHQUAKE_RT: EarthquakeTable =
    EarthquakeTable { premium_div: 2, cap: 25_000, old_full_until: 5_000, old_half_until: 15_000 };

/// 医療費控除の足切り：10万円と総所得金額等の5%の少ない方（所法73条）
pub const MEDICAL_THRESHOLD: i64 = 100_000;
pub const MEDICAL_THRESHOLD_INCOME_PCT: i64 = 5;
/// 医療費控除の上限
pub const MEDICAL_CAP: i64 = 2_000_000;
/// セルフメディケーション税制の足切りと上限（措法41条の17）
pub const SELF_MEDICATION_THRESHOLD: i64 = 12_000;
pub const SELF_MEDICATION_CAP: i64 = 88_000;

// ───────────── 寄附金・確定申告 ─────────────

/// 寄附金控除・寄附金税額控除の自己負担額
pub const DONATION_SELF_BURDEN: i64 = 2_000;
/// 所得税の寄附金控除の対象額の上限：総所得金額等の40%（所法78条）
pub const DONATION_IT_INCOME_LIMIT_PCT: i64 = 40;
/// 住民税の寄附金税額控除の対象額の上限：総所得金額等の30%（地方税法37条の2・314条の7）
pub const DONATION_RT_INCOME_LIMIT_PCT: i64 = 30;
/// ふるさと納税の特例控除額の上限：調整控除後の所得割額の20%
pub const FURUSATO_SPECIAL_CAP_PCT: i64 = 20;
/// ワンストップ特例を使える寄附先の数の上限
pub const ONE_STOP_MAX_DONEES: usize = 5;
/// ふるさと納税の上限額を探すときの刻み
pub const FURUSATO_SEARCH_STEP: i64 = 1_000;
/// 給与収入がこの額を超えると年末調整の対象外で、確定申告が必要（所法190条）
pub const SALARY_FILING_REQUIRED_OVER: i64 = 20_000_000;

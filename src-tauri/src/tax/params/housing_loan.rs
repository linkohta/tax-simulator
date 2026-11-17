//! 住宅借入金等特別控除（借入限度額、控除期間、住民税からの控除限度額）

/// 住宅借入金等特別控除の借入限度額と控除期間（1つの住宅の区分）
pub struct HousingLoanTerm {
    /// 借入限度額
    pub limit: i64,
    /// 特例対象個人（子育て世帯・若者夫婦世帯）の借入限度額。上乗せがなければ `limit` と同じ
    pub limit_child: i64,
    /// 控除期間（年）
    pub years: u32,
}

/// 万円単位で書く
const fn term(limit_man: i64, limit_child_man: i64, years: u32) -> HousingLoanTerm {
    HousingLoanTerm { limit: limit_man * 10_000, limit_child: limit_child_man * 10_000, years }
}

/// 令和4年以後に居住を開始した場合（措法41条。国税庁タックスアンサー No.1211-1〜3、令和8年4月1日現在）
pub struct HousingLoanPeriod {
    /// 居住開始年（西暦）の範囲
    pub from_year: u32,
    pub to_year: u32,
    /// 控除率（1/1000単位。0.7% = 7）
    pub rate_per_mille: i64,
    /// 合計所得金額の上限
    pub income_limit: i64,
    /// 新築・買取再販 [認定長期優良・低炭素, ZEH水準省エネ, 省エネ基準適合, その他]
    pub new: [HousingLoanTerm; 4],
    /// 中古（既存住宅）。並びは同じ
    pub existing: [HousingLoanTerm; 4],
}

/// 令和6年以後の「その他」の新築は、令和5年末までに建築確認を受けたものなどに限り 2,000万円・10年。
/// 令和10年以後の居住分は、令和8年分の計算には関係しないので載せていない。
pub static HOUSING_LOAN_PERIODS: &[HousingLoanPeriod] = &[
    HousingLoanPeriod {
        from_year: 2022,
        to_year: 2023,
        rate_per_mille: 7,
        income_limit: 20_000_000,
        new: [term(5_000, 5_000, 13), term(4_500, 4_500, 13), term(4_000, 4_000, 13), term(3_000, 3_000, 13)],
        existing: [term(3_000, 3_000, 10), term(3_000, 3_000, 10), term(3_000, 3_000, 10), term(2_000, 2_000, 10)],
    },
    HousingLoanPeriod {
        from_year: 2024,
        to_year: 2025,
        rate_per_mille: 7,
        income_limit: 20_000_000,
        new: [term(4_500, 5_000, 13), term(3_500, 4_500, 13), term(3_000, 4_000, 13), term(2_000, 2_000, 10)],
        existing: [term(3_000, 3_000, 10), term(3_000, 3_000, 10), term(3_000, 3_000, 10), term(2_000, 2_000, 10)],
    },
    // 令和8年度改正：省エネ性能の高い中古の限度額引上げ・上乗せ・13年化、省エネ基準適合の新築は2,000万円に
    HousingLoanPeriod {
        from_year: 2026,
        to_year: 2026,
        rate_per_mille: 7,
        income_limit: 20_000_000,
        new: [term(4_500, 5_000, 13), term(3_500, 4_500, 13), term(2_000, 3_000, 13), term(2_000, 2_000, 10)],
        existing: [term(3_500, 4_500, 13), term(3_500, 4_500, 13), term(2_000, 3_000, 13), term(2_000, 2_000, 10)],
    },
];
/// 新築の「その他の住宅」に建築確認の時期などの条件が付くようになった居住開始年
pub const HOUSING_LOAN_OTHER_NEW_RESTRICTED_FROM_YEAR: u32 = 2024;

/// 令和3年以前に居住を開始した場合（国税庁タックスアンサー No.1213 ほか）。
/// 平成28年以前の居住分は令和8年分では控除期間（10年）が終わっているので、平成29年からを載せる。
/// 控除期間13年の特例（消費税率10%での取得）も、令和8年分は10年目以内なので同じ算式になる。
pub struct HousingLoanLegacy {
    pub from_year: u32,
    pub to_year: u32,
    pub rate_per_mille: i64,
    pub income_limit: i64,
    pub years: u32,
    /// 特定取得（消費税率8%・10%）の借入限度額 [認定住宅の新築, それ以外]
    pub limit_specific: [i64; 2],
    /// 特定取得以外の借入限度額 [認定住宅の新築, それ以外]
    pub limit_other: [i64; 2],
}
pub static HOUSING_LOAN_LEGACY: HousingLoanLegacy = HousingLoanLegacy {
    from_year: 2017,
    to_year: 2021,
    rate_per_mille: 10,
    income_limit: 30_000_000,
    years: 10,
    limit_specific: [50_000_000, 40_000_000],
    limit_other: [30_000_000, 20_000_000],
};
/// この年以後に居住を開始すると、住民税の控除限度額で所得税の基礎控除の48万円超過分を加算しない
pub const HOUSING_LOAN_RT_NO_EXCESS_FROM_YEAR: u32 = 2026;

/// 住宅ローン控除の住民税からの控除限度額（所得税の課税総所得金額等×率、上限あり）
pub struct HousingLoanResidentLimit {
    /// 所得税の課税総所得金額等に掛ける率（%）
    pub rate_pct: i64,
    /// 控除限度額の上限
    pub cap: i64,
}
/// 原則：5%・97,500円（地方税法附則5条の4の2）
pub const HOUSING_LOAN_RT_STANDARD: HousingLoanResidentLimit = HousingLoanResidentLimit { rate_pct: 5, cap: 97_500 };
/// 平成26年4月〜令和3年12月入居で消費税率8%・10%の特定取得など：7%・136,500円（同条）
pub const HOUSING_LOAN_RT_SPECIFIC: HousingLoanResidentLimit = HousingLoanResidentLimit { rate_pct: 7, cap: 136_500 };

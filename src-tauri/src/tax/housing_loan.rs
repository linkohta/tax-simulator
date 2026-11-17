//! 住宅借入金等特別控除（控除額の計算と、住民税から控除する部分の限度額）

use super::fmt_yen;
use super::model::*;
use super::params::*;

/// 控除可能額と警告。手入力ならその額、自動計算なら年末残高などから求める
pub fn credit(hl: &HousingLoanInput, total_income: i64) -> (i64, Vec<String>) {
    match hl.mode {
        HousingLoanMode::Manual => (hl.credit_amount.max(0), Vec::new()),
        HousingLoanMode::Auto => auto_credit(hl, total_income),
    }
}

/// 居住年と住宅の区分から決まる条件
struct Terms {
    limit: i64,
    years: u32,
    rate_per_mille: i64,
    income_limit: i64,
}

fn terms(hl: &HousingLoanInput) -> Option<Terms> {
    let year = hl.move_in_year;
    let legacy = &HOUSING_LOAN_LEGACY;
    if (legacy.from_year..=legacy.to_year).contains(&year) {
        // 認定住宅の上乗せは新築だけ
        let certified = hl.housing_type == HousingType::Certified && !hl.existing_house;
        let limits = if hl.specific_acquisition { legacy.limit_specific } else { legacy.limit_other };
        return Some(Terms {
            limit: limits[if certified { 0 } else { 1 }],
            years: legacy.years,
            rate_per_mille: legacy.rate_per_mille,
            income_limit: legacy.income_limit,
        });
    }
    let period = HOUSING_LOAN_PERIODS.iter().find(|p| (p.from_year..=p.to_year).contains(&year))?;
    let index = match hl.housing_type {
        HousingType::Certified => 0,
        HousingType::Zeh => 1,
        HousingType::EnergySaving => 2,
        HousingType::Other => 3,
    };
    let term = if hl.existing_house { &period.existing[index] } else { &period.new[index] };
    Some(Terms {
        limit: if hl.child_rearing { term.limit_child } else { term.limit },
        years: term.years,
        rate_per_mille: period.rate_per_mille,
        income_limit: period.income_limit,
    })
}

/// 年末残高から控除可能額を計算する：min(年末残高, 借入限度額) × 控除率（100円未満切捨て）
fn auto_credit(hl: &HousingLoanInput, total_income: i64) -> (i64, Vec<String>) {
    let balance = hl.year_end_balance.max(0);
    if balance == 0 {
        return (0, Vec::new());
    }
    let year = hl.move_in_year;
    if year > TAX_YEAR {
        return (0, vec![format!("居住開始年が{TAX_YEAR}年より後のため、住宅借入金等特別控除は計算していません。")]);
    }
    let expired = || {
        (0, vec![format!("{year}年に居住を開始した場合、{TAX_YEAR}年分は控除期間が終わっているため、住宅借入金等特別控除は計算していません。")])
    };
    let Some(t) = terms(hl) else {
        return expired();
    };
    // 控除期間は居住を開始した年から数える
    if TAX_YEAR - year + 1 > t.years {
        return expired();
    }
    if total_income > t.income_limit {
        let w = format!("合計所得金額が{}円を超えるため、住宅借入金等特別控除は受けられません。", fmt_yen(t.income_limit));
        return (0, vec![w]);
    }
    let mut warnings = Vec::new();
    if !hl.existing_house && hl.housing_type == HousingType::Other && year >= HOUSING_LOAN_OTHER_NEW_RESTRICTED_FROM_YEAR {
        warnings.push(
            "省エネ基準に適合しない新築住宅は、2023年末までに建築確認を受けたものなどに限って控除できます（買取再販住宅を除く）。該当するものとして計算しています。"
                .into(),
        );
    }
    let amount = balance.min(t.limit) * t.rate_per_mille / 1000 / TAX_UNIT * TAX_UNIT;
    (amount, warnings)
}

/// 居住開始が令和8年以後か（住民税の限度額で、基礎控除の48万円超過分を加算しない）
fn moved_in_2026_or_later(hl: &HousingLoanInput) -> bool {
    match hl.mode {
        HousingLoanMode::Manual => hl.moved_in_2026_or_later,
        HousingLoanMode::Auto => hl.move_in_year >= HOUSING_LOAN_RT_NO_EXCESS_FROM_YEAR,
    }
}

/// 住民税の限度額が7%・136,500円になる入居か（令和3年以前の特定取得）
fn has_specific_resident_limit(hl: &HousingLoanInput) -> bool {
    match hl.mode {
        HousingLoanMode::Manual => hl.specific_acquisition && !hl.moved_in_2026_or_later,
        HousingLoanMode::Auto => hl.specific_acquisition && hl.move_in_year <= HOUSING_LOAN_LEGACY.to_year,
    }
}

/// 住民税から控除できる限度額（地方税法附則5条の4の2）
///
/// 所得税の課税総所得金額等 × 率（上限あり）。令和7年以前に居住を開始した場合は、
/// 所得税の基礎控除額のうち48万円を超える部分（`basic_excess`）を課税総所得金額等に加算する。
pub fn resident_limit(hl: &HousingLoanInput, taxable_total_it: i64, basic_excess: i64) -> i64 {
    let limit = if has_specific_resident_limit(hl) { &HOUSING_LOAN_RT_SPECIFIC } else { &HOUSING_LOAN_RT_STANDARD };
    let base = taxable_total_it + if moved_in_2026_or_later(hl) { 0 } else { basic_excess };
    (base.max(0) * limit.rate_pct / 100).min(limit.cap)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAN: i64 = 10_000;

    fn hl(moved_in_2026_or_later: bool, specific_acquisition: bool) -> HousingLoanInput {
        HousingLoanInput { credit_amount: 300_000, moved_in_2026_or_later, specific_acquisition, ..Default::default() }
    }

    fn auto(move_in_year: u32, housing_type: HousingType, balance_man: i64) -> HousingLoanInput {
        HousingLoanInput {
            mode: HousingLoanMode::Auto,
            year_end_balance: balance_man * MAN,
            move_in_year,
            housing_type,
            ..Default::default()
        }
    }

    fn amount(hl: &HousingLoanInput) -> i64 {
        credit(hl, 5_000_000).0
    }

    #[test]
    fn standard() {
        // (1,000,000 + 560,000) × 5% = 78,000
        assert_eq!(resident_limit(&hl(false, false), 1_000_000, 560_000), 78_000);
        // 上限の境界：1,950,000 × 5% = 97,500、それを超えても 97,500
        assert_eq!(resident_limit(&hl(false, false), 1_949_000, 0), 97_450);
        assert_eq!(resident_limit(&hl(false, false), 1_950_000, 0), 97_500);
        assert_eq!(resident_limit(&hl(false, false), 3_000_000, 0), 97_500);
    }

    #[test]
    fn moved_in_2026_or_later_ignores_basic_excess() {
        // 1,000,000 × 5% = 50,000（基礎控除の48万円超過分を加算しない）
        assert_eq!(resident_limit(&hl(true, false), 1_000_000, 560_000), 50_000);
        // 令和8年以後の入居に7%の区分はない
        assert_eq!(resident_limit(&hl(true, true), 1_000_000, 560_000), 50_000);
    }

    #[test]
    fn specific_acquisition() {
        // (1,000,000 + 560,000) × 7% = 109,200
        assert_eq!(resident_limit(&hl(false, true), 1_000_000, 560_000), 109_200);
        // 上限の境界：1,950,000 × 7% = 136,500
        assert_eq!(resident_limit(&hl(false, true), 1_949_000, 0), 136_430);
        assert_eq!(resident_limit(&hl(false, true), 1_950_000, 0), 136_500);
        assert_eq!(resident_limit(&hl(false, true), 3_000_000, 0), 136_500);
    }

    #[test]
    fn manual_amount_is_used_as_is() {
        assert_eq!(credit(&hl(false, false), 50_000_000), (300_000, Vec::new()));
    }

    #[test]
    fn new_house_2026() {
        // 借入限度額：認定 4,500万、ZEH 3,500万、省エネ 2,000万。控除率 0.7%
        assert_eq!(amount(&auto(2026, HousingType::Certified, 6_000)), 315_000);
        assert_eq!(amount(&auto(2026, HousingType::Zeh, 4_000)), 245_000);
        assert_eq!(amount(&auto(2026, HousingType::EnergySaving, 3_000)), 140_000);
        // 子育て世帯・若者夫婦世帯の上乗せ：5,000万、4,500万、3,000万
        let child = |t, b| HousingLoanInput { child_rearing: true, ..auto(2026, t, b) };
        assert_eq!(amount(&child(HousingType::Certified, 6_000)), 350_000);
        assert_eq!(amount(&child(HousingType::Zeh, 4_000)), 280_000);
        assert_eq!(amount(&child(HousingType::EnergySaving, 3_000)), 210_000);
        // 残高が限度額より少なければ残高 × 0.7%
        assert_eq!(amount(&auto(2026, HousingType::Zeh, 1_000)), 70_000);
        // 100円未満切捨て：12,345,678 × 0.7% = 86,419.7…
        let odd = HousingLoanInput { year_end_balance: 12_345_678, ..auto(2026, HousingType::Zeh, 0) };
        assert_eq!(amount(&odd), 86_400);
    }

    #[test]
    fn new_house_earlier_years() {
        // 令和6・7年：省エネ 3,000万（上乗せ 4,000万）
        assert_eq!(amount(&auto(2025, HousingType::EnergySaving, 3_500)), 210_000);
        assert_eq!(amount(&HousingLoanInput { child_rearing: true, ..auto(2025, HousingType::EnergySaving, 3_500) }), 245_000);
        assert_eq!(amount(&auto(2024, HousingType::Zeh, 4_000)), 245_000);
        // 令和4・5年：認定 5,000万、ZEH 4,500万、省エネ 4,000万、その他 3,000万（上乗せなし）
        assert_eq!(amount(&auto(2023, HousingType::Certified, 6_000)), 350_000);
        assert_eq!(amount(&auto(2022, HousingType::Zeh, 5_000)), 315_000);
        assert_eq!(amount(&auto(2023, HousingType::EnergySaving, 5_000)), 280_000);
        assert_eq!(amount(&auto(2023, HousingType::Other, 3_500)), 210_000);
        assert_eq!(amount(&HousingLoanInput { child_rearing: true, ..auto(2023, HousingType::Other, 3_500) }), 210_000);
        // 令和6年以後のその他の新築：2,000万（経過措置に当たるものとして計算し、警告する）
        let (a, w) = credit(&auto(2024, HousingType::Other, 3_500), 5_000_000);
        assert_eq!(a, 140_000);
        assert!(w[0].contains("建築確認"));
        assert!(credit(&auto(2023, HousingType::Other, 3_500), 5_000_000).1.is_empty());
    }

    #[test]
    fn existing_house() {
        let existing = |y, t, b| HousingLoanInput { existing_house: true, ..auto(y, t, b) };
        // 令和4〜7年：省エネ性能のある住宅 3,000万、その他 2,000万
        assert_eq!(amount(&existing(2022, HousingType::Zeh, 3_500)), 210_000);
        assert_eq!(amount(&existing(2025, HousingType::Certified, 3_500)), 210_000);
        assert_eq!(amount(&existing(2025, HousingType::Other, 2_500)), 140_000);
        // 令和8年：認定・ZEH 3,500万（上乗せ 4,500万）、省エネ 2,000万（上乗せ 3,000万）、その他 2,000万
        assert_eq!(amount(&existing(2026, HousingType::Certified, 5_000)), 245_000);
        assert_eq!(amount(&existing(2026, HousingType::Zeh, 5_000)), 245_000);
        assert_eq!(amount(&existing(2026, HousingType::EnergySaving, 5_000)), 140_000);
        assert_eq!(amount(&existing(2026, HousingType::Other, 5_000)), 140_000);
        let child = |t| HousingLoanInput { child_rearing: true, ..existing(2026, t, 5_000) };
        assert_eq!(amount(&child(HousingType::Zeh)), 315_000);
        assert_eq!(amount(&child(HousingType::EnergySaving)), 210_000);
        assert_eq!(amount(&child(HousingType::Other)), 140_000);
        // 中古のその他に、新築向けの警告は出さない
        assert!(credit(&existing(2026, HousingType::Other, 5_000), 5_000_000).1.is_empty());
    }

    #[test]
    fn legacy_years() {
        let specific = |y, t, b| HousingLoanInput { specific_acquisition: true, ..auto(y, t, b) };
        // 令和3年以前：控除率1%。特定取得は 4,000万（認定 5,000万）、それ以外は 2,000万（認定 3,000万）
        assert_eq!(amount(&specific(2021, HousingType::Other, 4_500)), 400_000);
        assert_eq!(amount(&specific(2021, HousingType::Certified, 6_000)), 500_000);
        assert_eq!(amount(&auto(2021, HousingType::Other, 2_500)), 200_000);
        assert_eq!(amount(&auto(2021, HousingType::Certified, 4_000)), 300_000);
        // 当時なかった区分（ZEH など）は一般の住宅として扱う。認定の上乗せは新築だけ
        assert_eq!(amount(&specific(2020, HousingType::Zeh, 4_500)), 400_000);
        assert_eq!(amount(&HousingLoanInput { existing_house: true, ..specific(2020, HousingType::Certified, 6_000) }), 400_000);
    }

    #[test]
    fn period_and_income_limits() {
        // 2017年入居は2026年分が10年目。2016年以前は終了
        assert_eq!(amount(&auto(2017, HousingType::Other, 1_000)), 100_000);
        let (a, w) = credit(&auto(2016, HousingType::Other, 1_000), 5_000_000);
        assert_eq!(a, 0);
        assert!(w[0].contains("控除期間が終わっている"));
        let (a, w) = credit(&auto(2027, HousingType::Zeh, 1_000), 5_000_000);
        assert_eq!(a, 0);
        assert!(w[0].contains("2026年より後"));
        // 合計所得金額：令和4年以後の入居は2,000万円、令和3年以前は3,000万円まで
        let zeh = auto(2026, HousingType::Zeh, 1_000);
        assert_eq!(credit(&zeh, 20_000_000).0, 70_000);
        assert_eq!(credit(&zeh, 20_000_001).0, 0);
        let old = auto(2021, HousingType::Other, 1_000);
        assert_eq!(credit(&old, 30_000_000).0, 100_000);
        assert_eq!(credit(&old, 30_000_001).0, 0);
        // 残高がなければ警告も出さない
        assert_eq!(credit(&auto(2016, HousingType::Other, 0), 5_000_000), (0, Vec::new()));
    }

    #[test]
    fn resident_limit_follows_move_in_year_in_auto_mode() {
        // 2026年入居：基礎控除の48万円超過分を加算しない
        assert_eq!(resident_limit(&auto(2026, HousingType::Zeh, 3_000), 1_000_000, 560_000), 50_000);
        assert_eq!(resident_limit(&auto(2025, HousingType::Zeh, 3_000), 1_000_000, 560_000), 78_000);
        // 2021年以前の特定取得：7%
        let specific = HousingLoanInput { specific_acquisition: true, ..auto(2021, HousingType::Other, 3_000) };
        assert_eq!(resident_limit(&specific, 1_000_000, 560_000), 109_200);
        // 2022年以後は、特定取得のチェックが残っていても5%
        let stale = HousingLoanInput { specific_acquisition: true, ..auto(2022, HousingType::Other, 3_000) };
        assert_eq!(resident_limit(&stale, 1_000_000, 560_000), 78_000);
    }
}

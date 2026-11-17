//! 住宅借入金等特別控除のうち、住民税から控除する部分

use super::model::HousingLoanInput;
use super::params::*;

/// 住民税から控除できる限度額（地方税法附則5条の4の2）
///
/// 所得税の課税総所得金額等 × 率（上限あり）。令和7年以前に居住を開始した場合は、
/// 所得税の基礎控除額のうち48万円を超える部分（`basic_excess`）を課税総所得金額等に加算する。
pub fn resident_limit(hl: &HousingLoanInput, taxable_total_it: i64, basic_excess: i64) -> i64 {
    let (limit, base) = if hl.moved_in_2026_or_later {
        (&HOUSING_LOAN_RT_STANDARD, taxable_total_it)
    } else if hl.specific_acquisition {
        (&HOUSING_LOAN_RT_SPECIFIC, taxable_total_it + basic_excess)
    } else {
        (&HOUSING_LOAN_RT_STANDARD, taxable_total_it + basic_excess)
    };
    (base.max(0) * limit.rate_pct / 100).min(limit.cap)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hl(moved_in_2026_or_later: bool, specific_acquisition: bool) -> HousingLoanInput {
        HousingLoanInput { credit_amount: 300_000, moved_in_2026_or_later, specific_acquisition }
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
}

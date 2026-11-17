//! 雑所得（公的年金等、副業）と、給与と年金の両方がある場合の所得金額調整控除

use super::model::MiscIncomeInput;
use super::params::*;

/// 雑所得の計算結果
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MiscIncome {
    /// 公的年金等に係る雑所得
    pub pension_net: i64,
    /// 副業の雑所得（収入 − 必要経費。赤字ならマイナス）
    pub side_net: i64,
    /// 雑所得の金額（公的年金等と副業を通算。赤字は0で、ほかの所得とは通算しない）
    pub total: i64,
    /// 所得金額調整控除（給与所得と年金所得の双方を有する者）。給与所得から引く
    pub salary_adjustment: i64,
}

/// 公的年金等に係る雑所得（1円未満切捨て）
///
/// `other_income` は公的年金等に係る雑所得以外の合計所得金額。1,000万円・2,000万円を超えると控除額が10万円ずつ減る。
pub fn pension_net(income: i64, age: u32, other_income: i64) -> i64 {
    if income <= 0 {
        return 0;
    }
    let table = if age >= PENSION_ELDERLY_AGE_FROM { PENSION_NET_65_AND_OVER } else { PENSION_NET_UNDER_65 };
    let tier = PENSION_OTHER_INCOME_TIERS.iter().filter(|limit| other_income > **limit).count() as i64;
    let Some(row) = table.iter().find(|r| income < r.below) else {
        return 0;
    };
    // 控除額が減る＝雑所得から引く額が減る
    let subtract = row.subtract - PENSION_DEDUCTION_REDUCTION_PER_TIER * tier;
    (income * row.rate_pct / 100 - subtract).max(0)
}

/// 所得金額調整控除（給与所得と年金所得の双方を有する者）
///
/// {給与所得（10万円まで）＋ 公的年金等に係る雑所得（10万円まで）} − 10万円。
/// `salary_net` は給与所得控除後の給与等の金額（子ども・特別障害者等の所得金額調整控除を引いた後）。
pub fn salary_pension_adjustment(salary_net: i64, pension_net: i64) -> i64 {
    let cap = SALARY_PENSION_ADJUSTMENT_CAP;
    (salary_net.clamp(0, cap) + pension_net.clamp(0, cap) - cap).max(0)
}

/// 雑所得と所得金額調整控除を計算する
///
/// `salary_net` は給与所得（年金との所得金額調整控除を引く前）、`other_income` は給与・雑所得以外の所得
/// （申告する配当や株式の譲渡所得）。公的年金等控除の区分の判定には、年金との所得金額調整控除を引く前の
/// 給与所得を使う（区分の境界で互いに依存しないようにするため）。
pub fn misc_income(m: &MiscIncomeInput, age: u32, salary_net: i64, other_income: i64) -> MiscIncome {
    let side_net = m.side_revenue.max(0) - m.side_expenses.max(0);
    let other_than_pension = salary_net + other_income + side_net.max(0);
    let pension_net = pension_net(m.pension_income, age, other_than_pension);
    MiscIncome {
        pension_net,
        side_net,
        total: (pension_net + side_net).max(0),
        salary_adjustment: salary_pension_adjustment(salary_net, pension_net),
    }
}

/// 雑所得があることで確定申告が必要になる理由と、申告についての注意
pub fn filing(m: &MiscIncomeInput, salary: i64, misc: &MiscIncome) -> (Option<&'static str>, Vec<String>) {
    if misc.total <= 0 {
        return (None, Vec::new());
    }
    if salary > 0 {
        if misc.total > FILING_OTHER_INCOME_OVER {
            return (Some("給与以外の所得20万円超"), Vec::new());
        }
        let note = "給与以外の所得が20万円以下なら所得税の確定申告は不要ですが（住民税の申告は必要）、ここでは所得税にも含めて計算しています。";
        return (None, vec![note.into()]);
    }
    // 給与がない年金受給者
    if m.pension_income > PENSION_FILING_EXEMPT_UNTIL {
        return (Some("公的年金等の収入400万円超"), Vec::new());
    }
    if misc.side_net > FILING_OTHER_INCOME_OVER {
        return (Some("公的年金等以外の所得20万円超"), Vec::new());
    }
    (None, Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pension_under_65() {
        let p = |income| pension_net(income, 64, 0);
        // 60万円以下は0、130万円未満は 収入 − 60万円
        assert_eq!((p(600_000), p(600_001), p(1_299_999)), (0, 1, 699_999));
        // 130万円以上410万円未満：×75% − 27.5万円
        assert_eq!(p(1_300_000), 700_000);
        // 4,099,999 × 75% = 3,074,999.25 → 切捨て − 275,000
        assert_eq!(p(4_099_999), 2_799_999);
        // 410万円以上770万円未満：×85% − 68.5万円
        assert_eq!((p(4_100_000), p(7_699_999)), (2_800_000, 5_859_999));
        // 770万円以上1,000万円未満：×95% − 145.5万円
        assert_eq!((p(7_700_000), p(9_999_999)), (5_860_000, 8_044_999));
        // 1,000万円以上：収入 − 195.5万円
        assert_eq!((p(10_000_000), p(12_000_000)), (8_045_000, 10_045_000));
        assert_eq!(p(0), 0);
    }

    #[test]
    fn pension_65_and_over() {
        let p = |income| pension_net(income, 65, 0);
        // 110万円以下は0、330万円未満は 収入 − 110万円
        assert_eq!((p(1_100_000), p(1_100_001), p(3_299_999)), (0, 1, 2_199_999));
        assert_eq!(p(3_300_000), 2_200_000);
        // 国税庁の計算例：350万円 × 75% − 27.5万円 = 235万円
        assert_eq!(p(3_500_000), 2_350_000);
        // 410万円以上は65歳未満と同じ
        assert_eq!(p(5_000_000), pension_net(5_000_000, 64, 0));
        // 年齢の境界
        assert_eq!(pension_net(2_000_000, 64, 0), 1_225_000);
        assert_eq!(pension_net(2_000_000, 65, 0), 900_000);
    }

    #[test]
    fn pension_other_income_tiers() {
        // ほかの所得が1,000万円以下：控除60万、1,000万円超：50万、2,000万円超：40万
        assert_eq!(pension_net(1_000_000, 60, 10_000_000), 400_000);
        assert_eq!(pension_net(1_000_000, 60, 10_000_001), 500_000);
        assert_eq!(pension_net(1_000_000, 60, 20_000_000), 500_000);
        assert_eq!(pension_net(1_000_000, 60, 20_000_001), 600_000);
        // 65歳以上・350万円：1,000万円超は ×75% − 17.5万円、2,000万円超は − 7.5万円
        assert_eq!(pension_net(3_500_000, 70, 10_000_001), 2_450_000);
        assert_eq!(pension_net(3_500_000, 70, 20_000_001), 2_550_000);
        // 0円になる収入の上限も下がる（65歳以上：100万円、90万円）
        assert_eq!((pension_net(1_000_000, 70, 10_000_001), pension_net(1_000_001, 70, 10_000_001)), (0, 1));
        assert_eq!((pension_net(900_000, 70, 20_000_001), pension_net(900_001, 70, 20_000_001)), (0, 1));
    }

    #[test]
    fn adjustment_for_salary_and_pension() {
        // どちらも10万円超：10万 + 10万 − 10万 = 10万
        assert_eq!(salary_pension_adjustment(500_000, 300_000), 100_000);
        // 5万 + 8万 − 10万 = 3万
        assert_eq!(salary_pension_adjustment(50_000, 80_000), 30_000);
        // 合計が10万円以下なら0
        assert_eq!(salary_pension_adjustment(50_000, 50_000), 0);
        assert_eq!(salary_pension_adjustment(60_000, 30_000), 0);
        // 片方しかなければ0
        assert_eq!(salary_pension_adjustment(0, 500_000), 0);
        assert_eq!(salary_pension_adjustment(500_000, 0), 0);
    }

    #[test]
    fn side_income_and_losses() {
        let m = |pension_income, side_revenue, side_expenses| MiscIncomeInput {
            pension_income,
            side_revenue,
            side_expenses,
        };
        // 副業：収入50万 − 経費20万 = 30万
        let r = misc_income(&m(0, 500_000, 200_000), 40, 3_000_000, 0);
        assert_eq!((r.pension_net, r.side_net, r.total, r.salary_adjustment), (0, 300_000, 300_000, 0));
        // 副業の赤字は年金の雑所得と通算できる：90万 − 20万 = 70万
        let r = misc_income(&m(2_000_000, 100_000, 300_000), 66, 0, 0);
        assert_eq!((r.pension_net, r.side_net, r.total), (900_000, -200_000, 700_000));
        // 雑所得全体が赤字でも0まで（ほかの所得とは通算しない）
        let r = misc_income(&m(0, 100_000, 300_000), 40, 3_000_000, 0);
        assert_eq!((r.side_net, r.total), (-200_000, 0));
        // 公的年金等控除の区分には副業の所得も入る：給与所得980万 + 副業30万 > 1,000万
        let r = misc_income(&m(1_000_000, 300_000, 0), 60, 9_800_000, 0);
        assert_eq!(r.pension_net, 500_000);
    }

    #[test]
    fn filing_reasons() {
        let m = |pension_income, side_revenue| MiscIncomeInput { pension_income, side_revenue, side_expenses: 0 };
        let check = |m: &MiscIncomeInput, salary: i64, age: u32| {
            let misc = misc_income(m, age, salary, 0);
            filing(m, salary, &misc)
        };
        // 給与あり：20万円の境界
        let (reason, notes) = check(&m(0, 200_000), 5_000_000, 40);
        assert_eq!(reason, None);
        assert!(notes[0].contains("20万円以下"));
        assert_eq!(check(&m(0, 200_001), 5_000_000, 40), (Some("給与以外の所得20万円超"), Vec::new()));
        // 給与なし：年金400万円以下で、ほかの所得20万円以下なら不要
        assert_eq!(check(&m(4_000_000, 200_000), 0, 70), (None, Vec::new()));
        assert_eq!(check(&m(4_000_001, 0), 0, 70).0, Some("公的年金等の収入400万円超"));
        assert_eq!(check(&m(3_000_000, 200_001), 0, 70).0, Some("公的年金等以外の所得20万円超"));
        // 雑所得がなければ何も出さない
        assert_eq!(check(&m(0, 0), 5_000_000, 40), (None, Vec::new()));
    }
}

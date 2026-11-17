//! 給与収入を変えたときの手取りと税・社会保険料（グラフ用）

use super::model::*;
use super::params::*;
use super::{compute, mul_div};

/// 給与収入を `salary` に置き換えた入力を作る。社会保険料の概算に使う月額給与と賞与（手入力なら金額）は、
/// 今の給与収入との比で伸び縮みさせる。今の給与収入が0なら、賞与なしで12等分する
fn with_salary(input: &TaxInput, salary: i64) -> TaxInput {
    let mut v = input.clone();
    let current = input.salary.annual_income.max(0);
    v.salary.annual_income = salary;
    let si = &mut v.social_insurance;
    if current > 0 {
        si.monthly_salary = mul_div(si.monthly_salary.max(0), salary, current);
        si.manual_amount = mul_div(si.manual_amount.max(0), salary, current);
        for b in si.bonuses.iter_mut() {
            *b = mul_div((*b).max(0), salary, current);
        }
    } else {
        si.monthly_salary = salary / MONTHS_PER_YEAR;
        si.bonuses.clear();
    }
    v
}

/// グラフの横軸の上限：今の給与収入の1.5倍（刻みに切上げ）。ただし下限と上限の範囲に収める
fn max_salary(current: i64) -> i64 {
    let step = SALARY_CURVE_STEP;
    let scaled = current.max(0) * SALARY_CURVE_RANGE_PCT / 100;
    ((scaled + step - 1) / step * step).clamp(SALARY_CURVE_MIN_RANGE, SALARY_CURVE_MAX_RANGE)
}

/// 給与収入を一定の刻みで変えたときの内訳。給与以外の入力（控除、株式、雑所得、寄附）はそのまま使う。
/// 0円の点は入れない（社会保険料の概算は最低等級の保険料を計上するので、給与0円では意味のある値にならない）
pub fn salary_curve(input: &TaxInput) -> Vec<CurvePoint> {
    let input = &input.normalized();
    let furusato_total: i64 = input.donations.furusato.iter().map(|e| e.amount.max(0)).sum();
    let one_stop = input.donations.one_stop;
    let max = max_salary(input.salary.annual_income);
    (1..=max / SALARY_CURVE_STEP)
        .map(|i| {
            let salary = i * SALARY_CURVE_STEP;
            let r = compute(&with_salary(input, salary), furusato_total, one_stop).result;
            CurvePoint {
                salary,
                take_home: r.summary.take_home,
                social_insurance: r.summary.social_insurance,
                // 申告不要とした株式の源泉徴収税は、所得税分・住民税分をそれぞれに含める
                income_tax: r.income_tax.total + r.stock_withholding.income_tax,
                resident_tax: r.resident_tax.total + r.stock_withholding.resident_tax,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tax::calculate;

    fn input(salary: i64) -> TaxInput {
        TaxInput {
            age: 35,
            salary: SalaryInput { annual_income: salary },
            social_insurance: SocialInsuranceInput {
                monthly_salary: 300_000,
                bonuses: vec![700_000, 700_000],
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn range() {
        // 今の1.5倍。ただし1,500万円以上、5,000万円以下
        assert_eq!(max_salary(0), 15_000_000);
        assert_eq!(max_salary(10_000_000), 15_000_000);
        assert_eq!(max_salary(10_000_001), 15_500_000);
        assert_eq!(max_salary(20_000_000), 30_000_000);
        assert_eq!(max_salary(100_000_000), 50_000_000);
        // 50万円刻みで、50万円から上限まで
        let curve = salary_curve(&input(5_000_000));
        assert_eq!(curve.len(), 30);
        assert_eq!((curve[0].salary, curve[1].salary, curve[29].salary), (500_000, 1_000_000, 15_000_000));
    }

    #[test]
    fn point_at_current_salary_matches_calculate() {
        let i = input(5_000_000);
        let r = calculate(&i);
        let p = salary_curve(&i).into_iter().find(|p| p.salary == 5_000_000).unwrap();
        assert_eq!(p.take_home, r.summary.take_home);
        assert_eq!(p.social_insurance, r.summary.social_insurance);
        assert_eq!((p.income_tax, p.resident_tax), (r.income_tax.total, r.resident_tax.total));
        // 内訳の合計は収入に一致する
        assert_eq!(p.take_home + p.social_insurance + p.income_tax + p.resident_tax, 5_000_000);
    }

    #[test]
    fn social_insurance_scales_with_salary() {
        // 給与を2倍にすると、月額30万・賞与70万×2 も2倍（60万・140万×2）で概算する
        let doubled = with_salary(&input(5_000_000), 10_000_000);
        assert_eq!(doubled.social_insurance.monthly_salary, 600_000);
        assert_eq!(doubled.social_insurance.bonuses, vec![1_400_000, 1_400_000]);
        // 今の給与が0なら、賞与なしで12等分
        let from_zero = with_salary(&input(0), 6_000_000);
        assert_eq!(from_zero.social_insurance.monthly_salary, 500_000);
        assert!(from_zero.social_insurance.bonuses.is_empty());
        // 手取りは給与が増えるほど増える
        let curve = salary_curve(&input(5_000_000));
        assert!(curve.windows(2).all(|w| w[1].take_home > w[0].take_home));
    }
}

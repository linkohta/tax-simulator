//! 給与からの社会保険料の概算（協会けんぽ・厚生年金・雇用保険）

use super::model::{SocialInsuranceBreakdown, SocialInsuranceInput, SocialInsuranceMode};
use super::params::*;

pub fn standard_monthly_health(monthly: i64) -> i64 {
    HEALTH_GRADES
        .iter()
        .rev()
        .find(|(_, lower)| monthly >= *lower)
        .map(|(s, _)| *s)
        .unwrap_or(HEALTH_GRADES[0].0)
}

/// 厚生年金の標準報酬月額（健保と等級境界が共通のため上下限でクランプ）
pub fn standard_monthly_pension(monthly: i64) -> i64 {
    standard_monthly_health(monthly).clamp(PENSION_STANDARD_MIN, PENSION_STANDARD_MAX)
}

/// 1円 = 1,000（端数処理のために 1/1000 円単位で計算する）
const MILLI: i128 = 1_000;
/// 50銭（1/1000 円単位）
const HALF_YEN: i128 = 500;

/// 1/1000 円単位の金額を「50銭以下切捨て・50銭超切上げ」で円にする
fn round_half_down(milli_yen: i128) -> i64 {
    (milli_yen / MILLI + if milli_yen % MILLI > HALF_YEN { 1 } else { 0 }) as i64
}

/// 被保険者負担分（保険料×料率÷2）を「50銭以下切捨て・50銭超切上げ」で円単位にする。
/// `rate` は 1/1000 % 単位（例: 9.91% → 9_910）。
fn employee_share(base: i64, rate: i64) -> i64 {
    // 労使折半なので分母は2倍
    round_half_down(base as i128 * rate as i128 * MILLI / (RATE_X1000_DENOM as i128 * 2))
}

/// 雇用保険料（労働者負担）。`rate_per_mille` は 1/1000 単位 × 10（例: 5/1000 → 50）
fn employment_share(wage: i64, rate: i64) -> i64 {
    round_half_down(wage as i128 * rate as i128 * MILLI / EMPLOYMENT_RATE_DENOM as i128)
}

pub fn estimate(input: &SocialInsuranceInput, age: u32) -> SocialInsuranceBreakdown {
    let other = input.other_amount.max(0);
    if input.mode == SocialInsuranceMode::Manual {
        let base = input.manual_amount.max(0);
        return SocialInsuranceBreakdown {
            other,
            total: base + other,
            health: base,
            estimated: false,
            ..Default::default()
        };
    }

    let monthly = input.monthly_salary.max(0);
    let std_h = standard_monthly_health(monthly);
    let std_p = standard_monthly_pension(monthly);
    let health_rate = kyokai_rate(&input.prefecture);
    let care_applies = (CARE_AGE_FROM..CARE_AGE_UNTIL).contains(&age);
    let support_months = input.childcare_support_months.min(12) as i64;

    let mut health = employee_share(std_h, health_rate) * 12;
    let mut care = if care_applies { employee_share(std_h, CARE_RATE) * 12 } else { 0 };
    let mut support = employee_share(std_h, CHILDCARE_SUPPORT_RATE) * support_months;
    let mut pension = employee_share(std_p, PENSION_RATE) * 12;

    // 賞与：標準賞与額（1,000円未満切捨て）。健保は年度累計上限、厚年は1回あたり上限
    let mut health_bonus_cum = 0;
    for &b in input.bonuses.iter().filter(|b| **b > 0) {
        let std_b = b / BONUS_STANDARD_UNIT * BONUS_STANDARD_UNIT;
        let h_b = std_b.min(HEALTH_BONUS_ANNUAL_CAP - health_bonus_cum).max(0);
        health_bonus_cum += h_b;
        let p_b = std_b.min(PENSION_BONUS_PER_PAYMENT_CAP);
        health += employee_share(h_b, health_rate);
        if care_applies {
            care += employee_share(h_b, CARE_RATE);
        }
        if support_months > 0 {
            // 賞与は支給月が不明のため、負担月数の割合で按分
            support += employee_share(h_b, CHILDCARE_SUPPORT_RATE) * support_months / 12;
        }
        pension += employee_share(p_b, PENSION_RATE);
    }

    // 年齢による被保険者資格：健康保険（子ども・子育て支援金を含む）は75歳未満、厚生年金は70歳未満
    if age >= HEALTH_AGE_UNTIL {
        health = 0;
        care = 0;
        support = 0;
    }
    if age >= PENSION_AGE_UNTIL {
        pension = 0;
    }

    let employment = if input.employment_insurance {
        // 月ごと・賞与ごとに端数処理
        employment_share(monthly, EMPLOYMENT_RATE) * 12
            + input
                .bonuses
                .iter()
                .filter(|b| **b > 0)
                .map(|&b| employment_share(b, EMPLOYMENT_RATE))
                .sum::<i64>()
    } else {
        0
    };

    let total = health + care + support + pension + employment + other;
    SocialInsuranceBreakdown {
        health,
        care,
        childcare_support: support,
        pension,
        employment,
        other,
        total,
        standard_monthly_health: std_h,
        standard_monthly_pension: std_p,
        estimated: true,
    }
}

fn kyokai_rate(pref: &str) -> i64 {
    let rate_of = |name: &str| KYOKAI_KENPO_RATES.iter().find(|(p, _)| *p == name).map(|(_, r)| *r);
    rate_of(pref).or_else(|| rate_of(KYOKAI_KENPO_FALLBACK_PREFECTURE)).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grades() {
        assert_eq!(standard_monthly_health(50_000), 58_000);
        assert_eq!(standard_monthly_health(300_000), 300_000);
        assert_eq!(standard_monthly_health(309_999), 300_000);
        assert_eq!(standard_monthly_health(310_000), 320_000);
        assert_eq!(standard_monthly_health(2_000_000), 1_390_000);
        assert_eq!(standard_monthly_pension(50_000), 88_000);
        assert_eq!(standard_monthly_pension(1_000_000), PENSION_STANDARD_MAX);
    }

    fn at_age(age: u32) -> SocialInsuranceBreakdown {
        let input = SocialInsuranceInput { monthly_salary: 300_000, bonuses: vec![700_000], ..Default::default() };
        estimate(&input, age)
    }

    #[test]
    fn age_limits() {
        // 東京都：健保 300,000×9.85%/2 = 14,775、賞与 700,000×9.85%/2 = 34,475
        let health = 14_775 * 12 + 34_475;
        // 厚年 300,000×18.3%/2 = 27,450、賞与 64,050
        let pension = 27_450 * 12 + 64_050;
        // 介護 300,000×1.62%/2 = 2,430、賞与 5,670
        let care = 2_430 * 12 + 5_670;
        // 雇用 1,500×12 + 3,500（年齢によらない）
        let employment = 1_500 * 12 + 3_500;

        // 介護保険：40歳以上65歳未満
        assert_eq!(at_age(39).care, 0);
        assert_eq!(at_age(40).care, care);
        assert_eq!(at_age(64).care, care);
        assert_eq!(at_age(65).care, 0);
        // 厚生年金：70歳未満
        assert_eq!(at_age(69).pension, pension);
        assert_eq!(at_age(70).pension, 0);
        assert_eq!(at_age(70).health, health);
        // 健康保険・子ども子育て支援金：75歳未満
        assert_eq!(at_age(74).health, health);
        assert!(at_age(74).childcare_support > 0);
        let s = at_age(75);
        assert_eq!((s.health, s.care, s.childcare_support, s.pension), (0, 0, 0, 0));
        assert_eq!(s.employment, employment);
        assert_eq!(s.total, employment);
    }

    #[test]
    fn bonus_caps() {
        let input = SocialInsuranceInput { monthly_salary: 300_000, bonuses: vec![3_000_000, 3_000_000], ..Default::default() };
        let s = estimate(&input, 35);
        // 健保：標準賞与額は年度累計573万円まで。1回目 300万×4.925% = 147,750、
        //       2回目は残り273万×4.925% = 134,452.5 → 50銭以下切捨てで 134,452
        assert_eq!(s.health, 14_775 * 12 + 147_750 + 134_452);
        // 厚年：1回あたり150万円まで。150万×9.15% = 137,250 が2回
        assert_eq!(s.pension, 27_450 * 12 + 137_250 * 2);
        // 雇用保険に上限はない：300万×5/1000 = 15,000 が2回
        assert_eq!(s.employment, 1_500 * 12 + 15_000 * 2);

        // 上限ちょうどと1,000円超
        let one = |b: i64| estimate(&SocialInsuranceInput { monthly_salary: 300_000, bonuses: vec![b], ..Default::default() }, 35);
        assert_eq!(one(1_500_000).pension, one(1_501_000).pension);
        assert_eq!(one(5_730_000).health, one(5_731_000).health);
        // 標準賞与額は1,000円未満切捨て
        assert_eq!(one(700_999).health, one(700_000).health);
    }

    #[test]
    fn rounding() {
        // 300,000 × 18.3% / 2 = 27,450
        assert_eq!(employee_share(300_000, 18_300), 27_450);
        // 50銭ちょうどは切捨て: 1 × 100% / 2 = 0.5
        assert_eq!(employee_share(1, 100_000), 0);
        assert_eq!(employee_share(3, 100_000), 1);
    }
}

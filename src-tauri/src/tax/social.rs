//! 給与からの社会保険料の概算（健康保険・厚生年金・雇用保険）

use super::model::{HealthPlan, SocialInsuranceBreakdown, SocialInsuranceInput, SocialInsuranceMode};
use super::params::*;

pub fn standard_monthly_health(monthly: i64) -> i64 {
    HEALTH_GRADES.iter().rev().find(|(_, lower)| monthly >= *lower).map(|(s, _)| *s).unwrap_or(HEALTH_GRADES[0].0)
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

/// 被保険者負担分（保険料×料率÷除数）を「50銭以下切捨て・50銭超切上げ」で円単位にする。
/// `rate` は 1/1000 % 単位（例: 9.91% → 9_910）。`divisor` は労使合計の料率なら2、本人負担分の料率なら1。
fn share(base: i64, rate: i64, divisor: i64) -> i64 {
    round_half_down(base as i128 * rate as i128 * MILLI / (RATE_X1000_DENOM as i128 * divisor as i128))
}

/// 労使折半の被保険者負担分
fn employee_share(base: i64, rate: i64) -> i64 {
    share(base, rate, EMPLOYER_SPLIT)
}

/// 健康保険・介護保険・子ども子育て支援金の料率（1/1000 %単位）と、本人負担分にするための除数
struct HealthRates {
    health: i64,
    care: i64,
    support: i64,
    divisor: i64,
}

fn health_rates(input: &SocialInsuranceInput) -> HealthRates {
    match input.health_plan {
        HealthPlan::Kyokai => HealthRates {
            health: kyokai_rate(&input.prefecture),
            care: CARE_RATE,
            support: CHILDCARE_SUPPORT_RATE,
            divisor: EMPLOYER_SPLIT,
        },
        // 健保組合などは労使折半とは限らないので、本人負担分の料率をそのまま使う
        HealthPlan::Custom => HealthRates {
            health: input.custom_health_rate_x1000.max(0),
            care: input.custom_care_rate_x1000.max(0),
            support: input.custom_childcare_support_rate_x1000.max(0),
            divisor: 1,
        },
    }
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
    let rates = health_rates(input);
    let hs = |base: i64, rate: i64| share(base, rate, rates.divisor);
    let care_applies = (CARE_AGE_FROM..CARE_AGE_UNTIL).contains(&age);
    let support_months = input.childcare_support_months.min(12) as i64;

    let mut health = hs(std_h, rates.health) * 12;
    let mut care = if care_applies { hs(std_h, rates.care) * 12 } else { 0 };
    let mut support = hs(std_h, rates.support) * support_months;
    let mut pension = employee_share(std_p, PENSION_RATE) * 12;

    // 賞与：標準賞与額（1,000円未満切捨て）。健保は年度累計上限、厚年は1回あたり上限
    let mut health_bonus_cum = 0;
    for &b in input.bonuses.iter().filter(|b| **b > 0) {
        let std_b = b / BONUS_STANDARD_UNIT * BONUS_STANDARD_UNIT;
        let h_b = std_b.min(HEALTH_BONUS_ANNUAL_CAP - health_bonus_cum).max(0);
        health_bonus_cum += h_b;
        let p_b = std_b.min(PENSION_BONUS_PER_PAYMENT_CAP);
        health += hs(h_b, rates.health);
        if care_applies {
            care += hs(h_b, rates.care);
        }
        if support_months > 0 {
            // 賞与は支給月が不明のため、負担月数の割合で按分
            support += hs(h_b, rates.support) * support_months / 12;
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
            + input.bonuses.iter().filter(|b| **b > 0).map(|&b| employment_share(b, EMPLOYMENT_RATE)).sum::<i64>()
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
    fn custom_health_plan() {
        let custom = |monthly_salary: i64, bonuses: Vec<i64>, health: i64| SocialInsuranceInput {
            monthly_salary,
            bonuses,
            health_plan: HealthPlan::Custom,
            custom_health_rate_x1000: health,
            custom_care_rate_x1000: 900,
            custom_childcare_support_rate_x1000: 100,
            ..Default::default()
        };
        let s = estimate(&custom(300_000, vec![700_000], 4_500), 45);
        // 本人負担分の料率をそのまま掛ける：健保 30万×4.5% = 13,500 ×12 + 賞与 70万×4.5% = 31,500
        assert_eq!(s.health, 13_500 * 12 + 31_500);
        // 介護 30万×0.9% = 2,700 ×12 + 6,300
        assert_eq!(s.care, 2_700 * 12 + 6_300);
        // 支援金 30万×0.1% = 300 ×8か月 + 賞与 700×8/12 = 466
        assert_eq!(s.childcare_support, 300 * 8 + 466);
        // 厚生年金・雇用保険は変わらない
        assert_eq!(s.pension, 27_450 * 12 + 64_050);
        assert_eq!(s.employment, 1_500 * 12 + 3_500);
        // 端数：98,000×4.925% = 4,826.5 → 50銭以下切捨てで 4,826
        assert_eq!(estimate(&custom(98_000, vec![], 4_925), 30).health, 4_826 * 12);
        // 年齢の区分は協会けんぽと同じ
        assert_eq!(estimate(&custom(300_000, vec![], 4_500), 39).care, 0);
        assert_eq!(estimate(&custom(300_000, vec![], 4_500), 75).health, 0);
    }

    #[test]
    fn custom_defaults_match_kyokai_tokyo() {
        // 初期値の料率（協会けんぽ東京都の半分）なら、協会けんぽ（東京都）と同じ結果になる
        let kyokai = SocialInsuranceInput { monthly_salary: 300_000, bonuses: vec![700_000], ..Default::default() };
        let custom = SocialInsuranceInput { health_plan: HealthPlan::Custom, ..kyokai.clone() };
        let (a, b) = (estimate(&kyokai, 45), estimate(&custom, 45));
        assert_eq!((a.health, a.care, a.childcare_support, a.total), (b.health, b.care, b.childcare_support, b.total));
    }

    #[test]
    fn bonus_caps() {
        let input =
            SocialInsuranceInput { monthly_salary: 300_000, bonuses: vec![3_000_000, 3_000_000], ..Default::default() };
        let s = estimate(&input, 35);
        // 健保：標準賞与額は年度累計573万円まで。1回目 300万×4.925% = 147,750、
        //       2回目は残り273万×4.925% = 134,452.5 → 50銭以下切捨てで 134,452
        assert_eq!(s.health, 14_775 * 12 + 147_750 + 134_452);
        // 厚年：1回あたり150万円まで。150万×9.15% = 137,250 が2回
        assert_eq!(s.pension, 27_450 * 12 + 137_250 * 2);
        // 雇用保険に上限はない：300万×5/1000 = 15,000 が2回
        assert_eq!(s.employment, 1_500 * 12 + 15_000 * 2);

        // 上限ちょうどと1,000円超
        let one = |b: i64| {
            estimate(&SocialInsuranceInput { monthly_salary: 300_000, bonuses: vec![b], ..Default::default() }, 35)
        };
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

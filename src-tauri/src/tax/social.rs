//! 給与からの社会保険料の概算（協会けんぽ・厚生年金・雇用保険）

use super::model::{SocialInsuranceBreakdown, SocialInsuranceInput, SocialInsuranceMode};
use super::params::*;

/// 健康保険の標準報酬月額表（等級ごとの「標準報酬月額」と「報酬月額の下限」）
const HEALTH_GRADES: [(i64, i64); 50] = [
    (58_000, 0),
    (68_000, 63_000),
    (78_000, 73_000),
    (88_000, 83_000),
    (98_000, 93_000),
    (104_000, 101_000),
    (110_000, 107_000),
    (118_000, 114_000),
    (126_000, 122_000),
    (134_000, 130_000),
    (142_000, 138_000),
    (150_000, 146_000),
    (160_000, 155_000),
    (170_000, 165_000),
    (180_000, 175_000),
    (190_000, 185_000),
    (200_000, 195_000),
    (220_000, 210_000),
    (240_000, 230_000),
    (260_000, 250_000),
    (280_000, 270_000),
    (300_000, 290_000),
    (320_000, 310_000),
    (340_000, 330_000),
    (360_000, 350_000),
    (380_000, 370_000),
    (410_000, 395_000),
    (440_000, 425_000),
    (470_000, 455_000),
    (500_000, 485_000),
    (530_000, 515_000),
    (560_000, 545_000),
    (590_000, 575_000),
    (620_000, 605_000),
    (650_000, 635_000),
    (680_000, 665_000),
    (710_000, 695_000),
    (750_000, 730_000),
    (790_000, 770_000),
    (830_000, 810_000),
    (880_000, 855_000),
    (930_000, 905_000),
    (980_000, 955_000),
    (1_030_000, 1_005_000),
    (1_090_000, 1_055_000),
    (1_150_000, 1_115_000),
    (1_210_000, 1_175_000),
    (1_270_000, 1_235_000),
    (1_330_000, 1_295_000),
    (1_390_000, 1_355_000),
];

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

/// 被保険者負担分（保険料×料率÷2）を「50銭以下切捨て・50銭超切上げ」で円単位にする。
/// `rate` は 1/1000 % 単位（例: 9.91% → 9_910）。
fn employee_share(base: i64, rate: i64) -> i64 {
    // base × rate / 100_000 / 2 を 1/1000 円精度で計算
    let milli_yen = base as i128 * rate as i128 / 200; // = base*rate/100000/2 * 1000
    let yen = milli_yen / 1000;
    let rem = milli_yen % 1000;
    (yen + if rem > 500 { 1 } else { 0 }) as i64
}

/// 雇用保険料（労働者負担）。`rate_per_mille` は 1/1000 単位 × 10（例: 5/1000 → 50）
fn employment_share(wage: i64, rate: i64) -> i64 {
    let milli_yen = wage as i128 * rate as i128 / 10; // wage*rate/10000 *1000
    let yen = milli_yen / 1000;
    let rem = milli_yen % 1000;
    (yen + if rem > 500 { 1 } else { 0 }) as i64
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
    let care_applies = (40..65).contains(&age);
    let support_months = input.childcare_support_months.min(12) as i64;

    let mut health = employee_share(std_h, health_rate) * 12;
    let mut care = if care_applies { employee_share(std_h, CARE_RATE) * 12 } else { 0 };
    let mut support = employee_share(std_h, CHILDCARE_SUPPORT_RATE) * support_months;
    let mut pension = employee_share(std_p, PENSION_RATE) * 12;

    // 賞与：標準賞与額（1,000円未満切捨て）。健保は年度累計上限、厚年は1回あたり上限
    let mut health_bonus_cum = 0;
    for &b in input.bonuses.iter().filter(|b| **b > 0) {
        let std_b = b / 1000 * 1000;
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
    KYOKAI_KENPO_RATES
        .iter()
        .find(|(p, _)| *p == pref)
        .map(|(_, r)| *r)
        .unwrap_or(KYOKAI_KENPO_RATES[12].1) // 東京都
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

    #[test]
    fn rounding() {
        // 300,000 × 18.3% / 2 = 27,450
        assert_eq!(employee_share(300_000, 18_300), 27_450);
        // 50銭ちょうどは切捨て: 1 × 100% / 2 = 0.5
        assert_eq!(employee_share(1, 100_000), 0);
        assert_eq!(employee_share(3, 100_000), 1);
    }
}

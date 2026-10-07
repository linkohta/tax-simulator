//! 統合テスト（手計算した計算例との照合）

use super::deductions::{salary_deduction, salary_net, Kind};
use super::model::*;
use super::*;

fn single(salary: i64, social: i64) -> TaxInput {
    TaxInput {
        age: 35,
        salary: SalaryInput { annual_income: salary },
        social_insurance: SocialInsuranceInput {
            mode: SocialInsuranceMode::Manual,
            manual_amount: social,
            ..Default::default()
        },
        ..Default::default()
    }
}

#[test]
fn salary_income_table() {
    assert_eq!(salary_net(700_000, Kind::IncomeTax), 0);
    assert_eq!(salary_net(1_000_000, Kind::IncomeTax), 260_000);
    assert_eq!(salary_net(1_360_000, Kind::IncomeTax), 620_000);
    // 措法29条の4第2項の端数表
    assert_eq!(salary_net(2_190_999, Kind::IncomeTax), 1_450_999);
    assert_eq!(salary_net(2_191_000, Kind::IncomeTax), 1_451_000);
    assert_eq!(salary_net(2_195_000, Kind::IncomeTax), 1_453_000);
    assert_eq!(salary_net(2_199_999, Kind::IncomeTax), 1_456_000);
    assert_eq!(salary_net(2_200_000, Kind::IncomeTax), 1_460_000);
    // 別表第五（4,000円刻み）
    assert_eq!(salary_net(5_001_999, Kind::IncomeTax), 3_560_000);
    assert_eq!(salary_deduction(10_000_000, Kind::Resident), 1_950_000);
}

#[test]
fn single_5m() {
    let r = calculate(&single(5_000_000, 750_000));
    assert_eq!(r.income.salary_net, 3_560_000);
    // 所得税：基礎控除104万 → 課税所得177万 × 5% = 88,500、復興税 1,858 → 90,300
    assert_eq!(r.income_tax.taxable_ordinary, 1_770_000);
    assert_eq!(r.income_tax.base_tax, 88_500);
    assert_eq!(r.income_tax.total, 90_300);
    // 住民税：基礎控除43万 → 課税所得238万、10% = 238,000、調整控除2,500 → 235,500 + 均等割5,000
    assert_eq!(r.resident_tax.taxable_ordinary, 2_380_000);
    assert_eq!(r.resident_tax.income_levy, 235_500);
    assert_eq!(r.resident_tax.total, 240_500);
}

#[test]
fn furusato_one_stop() {
    let mut input = single(5_000_000, 750_000);
    input.donations.furusato = vec![FurusatoEntry { name: "A".into(), amount: 50_000 }];
    input.donations.one_stop = true;
    let r = calculate(&input);
    // 人的控除差調整額 = 5万 + (104万 − 48万) = 61万 → 238万 − 61万 = 177万 → 特例割合 84.895%
    // 基本 4,800 + 特例 40,749 + 申告特例 2,450 ≒ 48,000
    assert_eq!(r.income_tax.total, 90_300, "ワンストップでは所得税は変わらない");
    assert!(r.furusato.current_burden <= 2_100, "burden = {}", r.furusato.current_burden);
    // 特例分の上限（所得割の20% = 47,100）から 47,100/0.84895 + 2,000 ≒ 57,480
    assert!((56_000..=58_000).contains(&r.furusato.limit), "limit = {}", r.furusato.limit);
}

#[test]
fn furusato_filing_matches_one_stop() {
    let mut a = single(8_000_000, 1_150_000);
    a.donations.furusato = vec![FurusatoEntry { name: "A".into(), amount: 80_000 }];
    a.donations.one_stop = true;
    let mut b = a.clone();
    b.donations.one_stop = false;
    let ra = calculate(&a);
    let rb = calculate(&b);
    assert!(rb.income_tax.total < ra.income_tax.total);
    let diff = (ra.furusato.current_burden - rb.furusato.current_burden).abs();
    assert!(diff <= 300, "ワンストップと確定申告の負担差 {diff}");
    assert!(rb.summary.filing_required);
}

#[test]
fn spouse_thresholds() {
    let mut input = single(6_000_000, 900_000);
    input.family.has_spouse = true;
    input.family.spouse.age = 40;
    input.family.spouse.salary_income = 1_360_000; // 所得62万 → 配偶者控除
    let r = calculate(&input);
    assert!(r.income_tax.deductions.iter().any(|d| d.label == "配偶者控除" && d.amount == 380_000));
    assert!(r.resident_tax.deductions.iter().any(|d| d.label == "配偶者控除" && d.amount == 330_000));

    input.family.spouse.salary_income = 1_700_000; // 所得96万 → 配偶者特別控除36万/33万
    let r = calculate(&input);
    assert!(r.income_tax.deductions.iter().any(|d| d.label == "配偶者特別控除" && d.amount == 360_000));
    assert!(r.resident_tax.deductions.iter().any(|d| d.label == "配偶者特別控除" && d.amount == 330_000));
}

#[test]
fn specific_relative_and_life_insurance_special() {
    let mut input = single(7_000_000, 1_000_000);
    input.family.dependents = vec![
        DependentInput { name: "大学生".into(), age: 20, salary_income: 1_800_000, ..Default::default() },
        DependentInput { name: "子".into(), age: 10, ..Default::default() },
    ];
    input.insurance.life_new = 120_000;
    let r = calculate(&input);
    // 1,800,000 → 所得 1,060,000 → 特定親族特別控除 21万（所得税・住民税）
    let find = |v: &Vec<DeductionItem>, l: &str| v.iter().find(|d| d.label.starts_with(l)).map(|d| d.amount);
    assert_eq!(find(&r.income_tax.deductions, "特定親族特別控除"), Some(210_000));
    assert_eq!(find(&r.resident_tax.deductions, "特定親族特別控除"), Some(210_000));
    // 23歳未満の扶養親族あり → 一般生命保険料（新）の上限6万（所得税のみ）
    assert_eq!(find(&r.income_tax.deductions, "生命保険料控除"), Some(60_000));
    assert_eq!(find(&r.resident_tax.deductions, "生命保険料控除"), Some(28_000));
}

#[test]
fn low_income_exemption() {
    // 給与110万：所得36万 ≤ 45万 → 住民税非課税
    let r = calculate(&single(1_100_000, 0));
    assert_eq!(r.resident_tax.total, 0);
    assert_eq!(r.income_tax.total, 0);
}

#[test]
fn stock_loss_offset() {
    let mut input = single(5_000_000, 750_000);
    input.stocks = StockInput {
        dividends: 300_000,
        dividend_mode: DividendMode::Separate,
        capital_gain: -500_000,
        declare_capital_gain: true,
        loss_carryforward: 0,
    };
    let r = calculate(&input);
    assert_eq!(r.income.dividend_separate, 0);
    assert_eq!(r.income.loss_carried_to_next_year, 200_000);
    assert_eq!(r.income_tax.tax_on_separate, 0);
}

#[test]
fn dividend_comprehensive_credit() {
    let mut input = single(5_000_000, 750_000);
    input.stocks.dividends = 200_000;
    input.stocks.dividend_mode = DividendMode::Comprehensive;
    let r = calculate(&input);
    assert!(r.income_tax.credits.iter().any(|c| c.label == "配当控除" && c.amount == 20_000));
    assert!(r.resident_tax.credits.iter().any(|c| c.label == "配当控除" && c.amount == 5_600));
}

#[test]
fn social_insurance_estimate() {
    let mut input = single(5_000_000, 0);
    input.social_insurance = SocialInsuranceInput {
        mode: SocialInsuranceMode::Auto,
        monthly_salary: 300_000,
        bonuses: vec![700_000],
        prefecture: "東京都".into(),
        employment_insurance: true,
        childcare_support_months: 8,
        ..Default::default()
    };
    let r = calculate(&input);
    let si = &r.social_insurance;
    // 健保 300,000×9.85%/2 = 14,775 ×12 + 賞与 700,000×9.85%/2 = 34,475
    assert_eq!(si.health, 14_775 * 12 + 34_475);
    // 厚年 27,450×12 + 64,050
    assert_eq!(si.pension, 27_450 * 12 + 64_050);
    // 雇用 1,500×12 + 3,500
    assert_eq!(si.employment, 1_500 * 12 + 3_500);
    assert_eq!(si.care, 0);
}

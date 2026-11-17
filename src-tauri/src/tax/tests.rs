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
    // 特別徴収：240,500 ÷ 12 → 7月以降 20,000、6月 20,500
    assert_eq!((r.resident_tax.monthly_first, r.resident_tax.monthly_rest), (20_500, 20_000));
    // 手取り 500万 − 社保75万 − 所得税 90,300 − 住民税 240,500 = 3,919,200 → 月平均 326,600
    assert_eq!((r.summary.take_home, r.summary.take_home_monthly), (3_919_200, 326_600));
}

#[test]
fn income_tax_brackets() {
    // 速算表の各区分の上端（課税所得は1,000円単位）と、次の区分の下端
    let t = progressive_income_tax;
    assert_eq!((t(1_949_000), t(1_950_000)), (97_450, 97_500));
    assert_eq!((t(3_299_000), t(3_300_000)), (232_400, 232_500));
    assert_eq!((t(6_949_000), t(6_950_000)), (962_300, 962_500));
    assert_eq!((t(8_999_000), t(9_000_000)), (1_433_770, 1_434_000));
    assert_eq!((t(17_999_000), t(18_000_000)), (4_403_670, 4_404_000));
    assert_eq!((t(39_999_000), t(40_000_000)), (13_203_600, 13_204_000));
    assert_eq!(t(0), 0);
}

#[test]
fn income_adjustment_with_child() {
    // 給与1,000万・23歳未満の扶養親族あり：所得金額調整控除 (1,000万 − 850万) × 10% = 15万
    let mut input = single(10_000_000, 1_400_000);
    input.family.dependents = vec![DependentInput { age: 10, ..Default::default() }];
    let r = calculate(&input);
    assert_eq!(r.income.income_adjustment_deduction, 150_000);
    // 給与所得 1,000万 − 195万 − 15万 = 790万
    assert_eq!(r.income.salary_net, 7_900_000);
    // 所得税：790万 − 社保140万 − 基礎控除62万 = 588万 × 20% − 427,500 = 748,500、復興税 15,718 → 764,200
    assert_eq!(r.income_tax.taxable_ordinary, 5_880_000);
    assert_eq!(r.income_tax.total, 764_200);
    // 子がいなければ調整控除なし
    assert_eq!(calculate(&single(10_000_000, 1_400_000)).income.income_adjustment_deduction, 0);
}

#[test]
fn adjustment_credit_under_2m() {
    // 給与300万 → 給与所得 300万 − (300万×30% + 8万) = 202万
    let r = calculate(&single(3_000_000, 450_000));
    assert_eq!(r.income.salary_net, 2_020_000);
    // 所得税：202万 − 45万 − 基礎控除104万 = 53万 × 5% = 26,500、復興税 556 → 27,000
    assert_eq!(r.income_tax.taxable_ordinary, 530_000);
    assert_eq!(r.income_tax.total, 27_000);
    // 住民税：202万 − 45万 − 43万 = 114万 × 10% = 114,000
    // 調整控除（課税所得200万円以下）：min(人的控除差5万, 114万) × 5% = 2,500
    assert_eq!(r.resident_tax.taxable_ordinary, 1_140_000);
    assert_eq!(r.resident_tax.income_levy, 111_500);
    assert_eq!(r.resident_tax.total, 116_500);
}

#[test]
fn adjustment_credit_over_2m_with_family() {
    // 給与600万 → 給与所得 600万 − (600万×20% + 44万) = 436万
    let mut input = single(6_000_000, 1_000_000);
    input.family.has_spouse = true;
    input.family.spouse.age = 40;
    input.family.dependents = vec![DependentInput { age: 20, ..Default::default() }];
    let r = calculate(&input);
    assert_eq!(r.income.salary_net, 4_360_000);
    // 所得税：436万 − 社保100万 − 基礎104万 − 配偶者38万 − 特定扶養63万 = 131万 × 5% = 65,500
    //         復興税 1,375 → 66,800
    assert_eq!(r.income_tax.taxable_ordinary, 1_310_000);
    assert_eq!(r.income_tax.total, 66_800);
    // 住民税：436万 − 100万 − 43万 − 33万 − 45万 = 215万 → 市 129,000・県 86,000
    // 調整控除（200万円超）：{人的控除差 28万（基礎5+配偶者5+特定18） − (215万 − 200万)} × 5% = 6,500
    //   → 市 3,900・県 2,600
    assert_eq!(r.resident_tax.taxable_ordinary, 2_150_000);
    assert!(r.resident_tax.credits.iter().any(|c| c.label == "調整控除" && c.amount == 6_500));
    assert_eq!(r.resident_tax.income_levy, 208_500);
}

#[test]
fn resident_tax_exemption_single() {
    // 扶養なし：合計所得45万円以下は均等割・所得割とも非課税。給与119万 − 74万 = 45万
    let r = calculate(&single(1_190_000, 0));
    assert!(r.resident_tax.per_capita_exempt && r.resident_tax.income_levy_exempt);
    assert_eq!(r.resident_tax.total, 0);
    // 1円超えると課税：課税所得 450,001 − 43万 = 2万 → 2,000 − 調整控除 1,000 = 1,000、均等割 5,000
    let r = calculate(&single(1_190_001, 0));
    assert!(!r.resident_tax.per_capita_exempt && !r.resident_tax.income_levy_exempt);
    assert_eq!((r.resident_tax.income_levy, r.resident_tax.per_capita), (1_000, 5_000));
    assert_eq!(r.income_tax.total, 0);
}

#[test]
fn resident_tax_exemption_with_family() {
    // 配偶者と子1人（本人を含め3人）：均等割の限度 35万×3+10万+21万 = 136万、所得割の限度 35万×3+10万+32万 = 147万
    let family = |salary: i64| {
        let mut input = single(salary, 0);
        input.family.has_spouse = true;
        input.family.spouse.age = 40;
        input.family.dependents = vec![DependentInput { age: 10, ..Default::default() }];
        calculate(&input).resident_tax
    };
    // 給与210万 − 74万 = 136万：非課税
    assert_eq!(family(2_100_000).total, 0);
    // 1円超えると均等割だけ課税
    let r = family(2_100_001);
    assert!(!r.per_capita_exempt && r.income_levy_exempt);
    assert_eq!(r.total, 5_000);
    // 給与221.2万 → 所得 1,468,400（147万以下）：所得割は非課税
    assert!(family(2_212_000).income_levy_exempt);
    // 給与221.6万 → 所得 1,471,200：所得割も課税
    //   課税所得 1,471,200 − 43万 − 配偶者33万 = 711,000 → 市 42,660・県 28,440
    //   調整控除 min(人的控除差10万, 71.1万) × 5% → 市 3,000・県 2,000
    //   市 39,660 → 39,600、県 26,440 → 26,400
    let r = family(2_216_000);
    assert!(!r.income_levy_exempt);
    assert_eq!((r.income_levy, r.total), (66_000, 71_000));
}

#[test]
fn resident_tax_exemption_for_disabled() {
    // 障害者・未成年者・寡婦・ひとり親は合計所得135万円以下で非課税。給与209万 − 74万 = 135万
    let disabled = |salary: i64| {
        let mut input = single(salary, 0);
        input.family.self_disability = Disability::General;
        calculate(&input).resident_tax
    };
    assert_eq!(disabled(2_090_000).total, 0);
    assert!(disabled(2_090_001).total > 0);
}

#[test]
fn other_donations() {
    // 所得税のみ対象の寄附10万：寄附金控除 98,000 → 課税所得 1,672,000 × 5% = 83,600、復興税 1,755 → 85,300
    let mut input = single(5_000_000, 750_000);
    input.donations.other_income_tax_only = 100_000;
    let r = calculate(&input);
    assert_eq!(r.income_tax.taxable_ordinary, 1_672_000);
    assert_eq!(r.income_tax.total, 85_300);
    assert_eq!(r.resident_tax.total, 240_500, "住民税は変わらない");
    assert!(r.summary.filing_required);

    // 条例指定の寄附10万：住民税の基本分 (10万 − 2,000) × 10% → 市 5,880・県 3,920
    //   市 141,300 − 5,880 = 135,420 → 135,400、県 94,200 − 3,920 = 90,280 → 90,200
    let mut input = single(5_000_000, 750_000);
    input.donations.other_resident_designated = 100_000;
    let r = calculate(&input);
    assert_eq!(r.income_tax.total, 85_300);
    assert_eq!(r.resident_tax.income_levy, 225_600);
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
        ..Default::default()
    };
    // 旧形式（合算の譲渡損益）の入力も、口座1つとして同じ結果になる
    let r = calculate(&input);
    assert_eq!(r.income.dividend_separate, 0);
    assert_eq!(r.income.loss_carried_to_next_year, 200_000);
    assert_eq!(r.income_tax.tax_on_separate, 0);
}

#[test]
fn legacy_stock_fields_are_migrated() {
    let json = r#"{"stocks":{"dividends":300000,"capitalGain":-500000,"declareCapitalGain":true}}"#;
    let input: TaxInput = serde_json::from_str(json).unwrap();
    let n = input.normalized();
    assert_eq!(n.stocks.accounts.len(), 1);
    let a = &n.stocks.accounts[0];
    assert_eq!((a.kind, a.capital_gain, a.declare), (StockAccountKind::Withholding, -500_000, true));
    // 保存し直すと旧形式の項目は出力されない
    let saved = serde_json::to_string(&n).unwrap();
    assert!(!saved.contains("declareCapitalGain"));
    assert!(saved.contains("\"accounts\""));
}

#[test]
fn stock_options_compare_dividend_modes() {
    let mut input = single(5_000_000, 750_000);
    input.stocks.dividends = 200_000;
    let r = calculate(&input);
    let total = |mode: DividendMode| r.stock_options.iter().find(|o| o.dividend_mode == mode).unwrap();
    assert_eq!(r.stock_options.len(), 3);
    // 申告不要：所得税 90,300 + 住民税 240,500 + 源泉 20万×20.315% = 40,630
    let no = total(DividendMode::NoDeclare);
    assert_eq!(no.total_tax, 90_300 + 240_500 + 40_630);
    assert!(no.is_current && !no.is_best);
    assert_eq!(no.diff_from_current, 0);
    // 総合課税：所得税 課税197万 → 99,500 − 配当控除 20,000 = 79,500、復興税 1,669 → 81,100
    //           住民税 課税258万 → 258,000 − 調整控除 2,500 − 配当控除 5,600 = 249,900 + 5,000
    let comp = total(DividendMode::Comprehensive);
    assert_eq!((comp.income_tax, comp.resident_tax, comp.stock_withholding), (81_100, 254_900, 0));
    assert!(comp.is_best);
    assert_eq!(comp.diff_from_current, 336_000 - 371_430);
    // 申告分離：所得税 88,500 + 30,000 = 118,500、復興税 2,488 → 120,900
    //           住民税 238,000 + 10,000 − 2,500 = 245,500 + 5,000
    let sep = total(DividendMode::Separate);
    assert_eq!((sep.income_tax, sep.resident_tax), (120_900, 250_500));
    assert!(!sep.is_best);

    // 株式の入力がなければ比較しない
    assert!(calculate(&single(5_000_000, 750_000)).stock_options.is_empty());
}

#[test]
fn stock_options_include_declaring_withholding_accounts() {
    let mut input = single(5_000_000, 750_000);
    input.stocks.dividends = 300_000;
    input.stocks.accounts = vec![StockAccount {
        name: "A証券".into(),
        kind: StockAccountKind::Withholding,
        capital_gain: -500_000,
        declare: false,
        dividends_received: 0,
    }];
    let r = calculate(&input);
    // 配当3方式 × 口座を申告する/しない
    assert_eq!(r.stock_options.len(), 6);
    let best = r.stock_options.iter().find(|o| o.is_best).unwrap();
    // 損失を申告して配当を申告分離にすると、配当30万が損失と相殺されて株式の税がなくなり、20万を繰り越せる
    assert_eq!(best.dividend_mode, DividendMode::Separate);
    assert_eq!(best.declare_withholding_accounts, Some(true));
    assert_eq!(best.total_tax, 90_300 + 240_500);
    assert_eq!(best.loss_carried_to_next_year, 200_000);
    assert_eq!(r.stock_options.iter().filter(|o| o.is_current).count(), 1);
}

#[test]
fn stock_options_with_dividends_received_in_account() {
    // 配当30万をすべて、損失50万の源泉徴収ありの特定口座で受け取っている
    let mut input = single(5_000_000, 750_000);
    input.stocks.dividends = 300_000;
    input.stocks.accounts = vec![StockAccount {
        name: "A証券".into(),
        kind: StockAccountKind::Withholding,
        capital_gain: -500_000,
        declare: false,
        dividends_received: 300_000,
    }];
    let r = calculate(&input);
    // 何も申告しなくても口座内で通算され、株式の税はかからない
    assert_eq!(r.summary.stock_withholding, 0);
    assert_eq!(r.summary.total_tax, 90_300 + 240_500);
    // 「配当は申告不要・損失は申告」は選べない組み合わせなので、比較は5通り
    assert_eq!(r.stock_options.len(), 5);
    assert!(!r
        .stock_options
        .iter()
        .any(|o| o.dividend_mode == DividendMode::NoDeclare && o.declare_withholding_accounts == Some(true)));
    // 税額は現在と同じだが、申告分離で損失を申告すれば残り20万を翌年に繰り越せるので、そちらが最も有利
    let best = r.stock_options.iter().find(|o| o.is_best).unwrap();
    assert_eq!((best.dividend_mode, best.declare_withholding_accounts), (DividendMode::Separate, Some(true)));
    assert_eq!((best.diff_from_current, best.loss_carried_to_next_year), (0, 200_000));
}

#[test]
fn foreign_tax_credit_on_separate_dividends() {
    let mut input = single(5_000_000, 750_000);
    input.stocks.dividends_no_credit = 100_000; // 外国株式の配当
    input.stocks.dividend_mode = DividendMode::Separate;
    input.stocks.foreign_income = 100_000;
    input.stocks.foreign_tax_paid = 10_000;
    let r = calculate(&input);
    // 所得税 88,500 + 10万×15% = 103,500、復興税 2,173。合計所得 366万
    // 控除限度額：所得税 2,827、復興税 59、道府県民税 339、市町村民税 508
    assert_eq!(r.income_tax.base_tax, 103_500);
    assert_eq!(r.income_tax.foreign_tax_credit, 2_827 + 59);
    // 103,500 + 2,173 − 2,886 = 102,787 → 102,700
    assert_eq!(r.income_tax.total, 102_700);
    // 住民税：市 145,800 − 1,500 − 508 = 143,792 → 143,700、県 97,200 − 1,000 − 339 = 95,861 → 95,800
    assert_eq!(r.resident_tax.income_levy, 143_700 + 95_800);
    assert!(r.resident_tax.credits.iter().any(|c| c.label == "外国税額控除" && c.amount == 847));
    assert!(r.warnings.iter().any(|w| w.contains("6,267円")));
    assert!(r.summary.filing_required);
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
fn housing_loan_spills_over_to_resident_tax() {
    let mut input = single(5_000_000, 750_000);
    input.housing_loan.credit_amount = 200_000;
    let r = calculate(&input);
    // 所得税 88,500 を全額控除 → 残り 111,500 を住民税から
    assert_eq!(r.income_tax.total, 0);
    // 限度額：(課税総所得 177万 + 基礎控除の48万円超過分 56万) × 5% = 116,500 → 上限 97,500
    // 所得割 235,500 − 97,500 = 138,000
    assert_eq!(r.resident_tax.income_levy, 138_000);

    // 特定取得：233万 × 7% = 163,100 → 上限 136,500。残り 111,500 を全額控除
    input.housing_loan.specific_acquisition = true;
    let r = calculate(&input);
    assert_eq!(r.resident_tax.income_levy, 235_500 - 111_500);
}

#[test]
fn housing_loan_auto_calculation() {
    // 2026年入居・ZEH水準の新築・年末残高3,000万：3,000万 × 0.7% = 210,000
    let mut input = single(5_000_000, 750_000);
    input.housing_loan = HousingLoanInput {
        mode: HousingLoanMode::Auto,
        year_end_balance: 30_000_000,
        move_in_year: 2026,
        housing_type: HousingType::Zeh,
        ..Default::default()
    };
    let r = calculate(&input);
    // 所得税 88,500 を全額控除 → 残り 121,500
    assert_eq!(r.income_tax.total, 0);
    // 住民税の限度額：2026年入居なので基礎控除の超過分を加算せず、課税総所得 177万 × 5% = 88,500
    // 所得割 235,500 − 88,500 = 147,000
    assert_eq!(r.resident_tax.income_levy, 147_000);
    // 手入力の欄に値が残っていても、自動計算では使わない
    input.housing_loan.credit_amount = 999_999;
    assert_eq!(calculate(&input).resident_tax.income_levy, 147_000);
}

#[test]
fn deduction_savings() {
    // 入力がなければ空
    assert!(calculate(&single(5_000_000, 750_000)).savings.is_empty());

    // iDeCo 27.6万：
    //   所得税 課税所得 177万 → 149.4万 × 5% = 74,700、復興税 1,568 → 76,200（90,300 から 14,100 減）
    //   住民税 課税所得 238万 → 210.4万 → 市 126,240 − 1,500 = 124,740 → 124,700、
    //          県 84,160 − 1,000 = 83,160 → 83,100。計 207,800（235,500 から 27,700 減）
    let mut input = single(5_000_000, 750_000);
    input.pension_savings.ideco = 276_000;
    let r = calculate(&input);
    assert_eq!(r.savings.len(), 1);
    let s = &r.savings[0];
    assert_eq!(s.label, "iDeCo・小規模企業共済等");
    assert_eq!((s.income_tax, s.resident_tax, s.total), (14_100, 27_700, 41_800));

    // 生命保険料と地震保険料は別々に出る。足切り以下の医療費は出ない
    input.insurance.life_new = 80_000;
    input.insurance.earthquake = 20_000;
    input.medical.expenses = 50_000;
    let labels: Vec<String> = calculate(&input).savings.into_iter().map(|s| s.label).collect();
    assert_eq!(labels, ["iDeCo・小規模企業共済等", "生命保険料控除", "地震保険料控除"]);

    // 住宅ローン控除 20万：所得税 88,500 と住民税 97,500 が減る（housing_loan_spills_over_to_resident_tax と同じ例）
    let mut input = single(5_000_000, 750_000);
    input.housing_loan.credit_amount = 200_000;
    let r = calculate(&input);
    let s = &r.savings[0];
    assert_eq!((s.label.as_str(), s.income_tax, s.resident_tax), ("住宅ローン控除", 90_300, 97_500));
}

#[test]
fn minimum_tax_warning_threshold() {
    // 給与所得 = 収入 − 195万。基準所得金額 3.3億円ちょうどでは警告しない
    let warned = |salary: i64| calculate(&single(salary, 0)).warnings.iter().any(|w| w.contains("負担の適正化措置"));
    assert!(!warned(331_950_000));
    assert!(warned(331_950_001));
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

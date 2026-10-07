//! 所得控除（所得税・住民税）と人的控除差の計算

use super::model::*;
use super::params::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    IncomeTax,
    Resident,
}

/// 給与所得（所得金額調整控除前）
pub fn salary_net(income: i64, kind: Kind) -> i64 {
    let min = match kind {
        Kind::IncomeTax => SALARY_DEDUCTION_MIN_IT,
        Kind::Resident => SALARY_DEDUCTION_MIN_RT,
    };
    if income <= 0 {
        return 0;
    }
    if let Some(&(_, _, net)) = SALARY_NET_SPECIAL.iter().find(|(lo, hi, _)| income >= *lo && income < *hi) {
        return net;
    }
    // 収入660万円未満は所得税法別表第五（4,000円刻み）による
    let a = if income < 6_600_000 { income / 4_000 * 4_000 } else { income };
    let formula = if a <= 1_800_000 {
        a * 40 / 100 - 100_000
    } else if a <= 3_600_000 {
        a * 30 / 100 + 80_000
    } else if a <= 6_600_000 {
        a * 20 / 100 + 440_000
    } else if a <= 8_500_000 {
        a * 10 / 100 + 1_100_000
    } else {
        1_950_000
    };
    if formula <= min {
        (income - min).max(0)
    } else {
        a - formula
    }
}

/// 給与所得控除額（収入−給与所得）
pub fn salary_deduction(income: i64, kind: Kind) -> i64 {
    income.max(0) - salary_net(income, kind)
}

/// 親族の合計所得金額（給与＋その他）
pub fn relative_income(salary: i64, other: i64) -> i64 {
    // 親族の合計所得は所得税ベースで判定（住民税も同じ要件）
    salary_net(salary, Kind::IncomeTax) + other.max(0)
}

fn lookup(table: &[(i64, i64)], x: i64) -> i64 {
    table.iter().find(|(upper, _)| x <= *upper).map(|(_, v)| *v).unwrap_or(0)
}

pub fn basic_deduction(total_income: i64, kind: Kind) -> i64 {
    match kind {
        Kind::IncomeTax => lookup(BASIC_DEDUCTION_IT, total_income),
        Kind::Resident => lookup(BASIC_DEDUCTION_RT, total_income),
    }
}

/// 納税者の合計所得による区分（0: 900万以下, 1: 950万以下, 2: 1000万以下, None: 超）
fn taxpayer_tier(total_income: i64) -> Option<usize> {
    if total_income <= 9_000_000 {
        Some(0)
    } else if total_income <= 9_500_000 {
        Some(1)
    } else if total_income <= 10_000_000 {
        Some(2)
    } else {
        None
    }
}

#[derive(Debug, Clone, Default)]
pub struct PersonalDeductions {
    pub items: Vec<DeductionItem>,
    pub total: i64,
    /// 調整控除・ふるさと納税特例控除割合の判定に使う人的控除差の合計（基礎控除分を含む）
    pub personal_diff: i64,
    /// 住民税の非課税判定に使う人数（本人＋同一生計配偶者＋扶養親族）
    pub household_count: i64,
    pub has_dependents_for_exemption: bool,
    /// 23歳未満の扶養親族（生命保険料控除の特例・所得金額調整控除の判定）
    pub has_dependent_under_23: bool,
    pub has_special_disability_relative: bool,
}

pub struct FamilyContext<'a> {
    pub family: &'a FamilyInput,
    pub total_income: i64,
}

impl FamilyContext<'_> {
    /// 人的控除（基礎・配偶者・扶養・特定親族・障害者・寡婦/ひとり親・勤労学生）
    pub fn personal(&self, kind: Kind) -> PersonalDeductions {
        let it = kind == Kind::IncomeTax;
        let pick = |a: i64, b: i64| if it { a } else { b };
        let f = self.family;
        let ti = self.total_income;
        let mut out = PersonalDeductions { household_count: 1, ..Default::default() };
        let push = |out: &mut PersonalDeductions, label: &str, amount: i64| {
            if amount > 0 {
                out.items.push(DeductionItem { label: label.into(), amount });
                out.total += amount;
            }
        };

        // 基礎控除
        push(&mut out, "基礎控除", basic_deduction(ti, kind));
        out.personal_diff += PERSONAL_DIFF_BASIC;

        // 配偶者
        if f.has_spouse {
            let sp = &f.spouse;
            let sp_income = relative_income(sp.salary_income, sp.other_income);
            let elderly = sp.age >= 70;
            if sp_income <= DEPENDENT_INCOME_LIMIT {
                out.household_count += 1;
                out.has_dependents_for_exemption = true;
                if let Some(t) = taxpayer_tier(ti) {
                    let amt = if elderly {
                        pick(SPOUSE_ELDERLY_IT[t], SPOUSE_ELDERLY_RT[t])
                    } else {
                        pick(SPOUSE_IT[t], SPOUSE_RT[t])
                    };
                    push(&mut out, if elderly { "配偶者控除（老人）" } else { "配偶者控除" }, amt);
                    out.personal_diff += if elderly { SPOUSE_DIFF_ELDERLY[t] } else { SPOUSE_DIFF[t] };
                }
                match sp.disability {
                    Disability::None => {}
                    Disability::General => {
                        push(&mut out, "障害者控除（配偶者）", pick(DISABLED_IT, DISABLED_RT));
                        out.personal_diff += DISABLED_IT - DISABLED_RT;
                    }
                    Disability::Special => {
                        push(&mut out, "特別障害者控除（配偶者）", pick(SPECIAL_DISABLED_IT, SPECIAL_DISABLED_RT));
                        out.personal_diff += SPECIAL_DISABLED_IT - SPECIAL_DISABLED_RT;
                    }
                    Disability::SpecialCohabiting => {
                        push(&mut out, "同居特別障害者控除（配偶者）", pick(COHAB_SPECIAL_DISABLED_IT, COHAB_SPECIAL_DISABLED_RT));
                        out.personal_diff += COHAB_SPECIAL_DISABLED_IT - COHAB_SPECIAL_DISABLED_RT;
                        out.has_special_disability_relative = true;
                    }
                }
                if matches!(sp.disability, Disability::Special) {
                    out.has_special_disability_relative = true;
                }
            } else if let Some(t) = taxpayer_tier(ti) {
                if let Some(row) = SPOUSE_SPECIAL.iter().find(|r| sp_income <= r.upper) {
                    let amt = pick(row.it[t], row.rt[t]);
                    push(&mut out, "配偶者特別控除", amt);
                    out.personal_diff += row.diff[t];
                }
            }
        }

        // 扶養親族・特定親族
        for (i, d) in f.dependents.iter().enumerate() {
            let inc = relative_income(d.salary_income, d.other_income);
            let label_name = if d.name.trim().is_empty() { format!("親族{}", i + 1) } else { d.name.clone() };
            if d.age < 23 && inc <= DEPENDENT_INCOME_LIMIT {
                out.has_dependent_under_23 = true;
            }
            if inc <= DEPENDENT_INCOME_LIMIT {
                out.household_count += 1;
                out.has_dependents_for_exemption = true;
                let (label, amt_it, amt_rt, diff) = if d.age < 16 {
                    ("年少扶養（控除なし）", 0, 0, 0)
                } else if (19..=22).contains(&d.age) {
                    ("扶養控除（特定）", DEP_SPECIFIC_IT, DEP_SPECIFIC_RT, DIFF_DEP_SPECIFIC)
                } else if d.age >= 70 && d.cohabiting_parent {
                    ("扶養控除（同居老親等）", DEP_ELDERLY_COHAB_IT, DEP_ELDERLY_COHAB_RT, DIFF_DEP_ELDERLY_COHAB)
                } else if d.age >= 70 {
                    ("扶養控除（老人）", DEP_ELDERLY_IT, DEP_ELDERLY_RT, DIFF_DEP_ELDERLY)
                } else {
                    ("扶養控除（一般）", DEP_GENERAL_IT, DEP_GENERAL_RT, DIFF_DEP_GENERAL)
                };
                push(&mut out, &format!("{label}：{label_name}"), pick(amt_it, amt_rt));
                out.personal_diff += diff;
            } else if (19..=22).contains(&d.age) {
                if let Some(row) = SPECIFIC_RELATIVE.iter().find(|r| inc <= r.upper) {
                    push(&mut out, &format!("特定親族特別控除：{label_name}"), pick(row.it, row.rt));
                    out.personal_diff += row.diff;
                }
            }
            // 障害者控除は扶養控除の対象外（16歳未満）でも適用。所得要件は扶養親族と同じ
            if inc <= DEPENDENT_INCOME_LIMIT {
                let (label, a_it, a_rt) = match d.disability {
                    Disability::None => continue,
                    Disability::General => ("障害者控除", DISABLED_IT, DISABLED_RT),
                    Disability::Special => ("特別障害者控除", SPECIAL_DISABLED_IT, SPECIAL_DISABLED_RT),
                    Disability::SpecialCohabiting => ("同居特別障害者控除", COHAB_SPECIAL_DISABLED_IT, COHAB_SPECIAL_DISABLED_RT),
                };
                if matches!(d.disability, Disability::Special | Disability::SpecialCohabiting) {
                    out.has_special_disability_relative = true;
                }
                push(&mut out, &format!("{label}：{label_name}"), pick(a_it, a_rt));
                out.personal_diff += a_it - a_rt;
            }
        }

        // 本人の障害
        match f.self_disability {
            Disability::None => {}
            Disability::General => {
                push(&mut out, "障害者控除（本人）", pick(DISABLED_IT, DISABLED_RT));
                out.personal_diff += DIFF_DISABLED;
            }
            Disability::Special | Disability::SpecialCohabiting => {
                push(&mut out, "特別障害者控除（本人）", pick(SPECIAL_DISABLED_IT, SPECIAL_DISABLED_RT));
                out.personal_diff += DIFF_SPECIAL_DISABLED;
            }
        }

        // 寡婦・ひとり親（合計所得500万円以下）
        if ti <= WIDOW_INCOME_LIMIT {
            match f.widow_status {
                WidowStatus::None => {}
                WidowStatus::Widow => {
                    push(&mut out, "寡婦控除", pick(WIDOW_IT, WIDOW_RT));
                    out.personal_diff += DIFF_WIDOW;
                }
                WidowStatus::SingleMother | WidowStatus::SingleFather => {
                    push(&mut out, "ひとり親控除", pick(SINGLE_PARENT_IT, SINGLE_PARENT_RT));
                    out.personal_diff += if f.widow_status == WidowStatus::SingleMother {
                        DIFF_SINGLE_PARENT_MOTHER
                    } else {
                        DIFF_SINGLE_PARENT_FATHER
                    };
                }
            }
        }

        // 勤労学生
        if f.working_student && ti <= WORKING_STUDENT_INCOME_LIMIT {
            push(&mut out, "勤労学生控除", pick(WORKING_STUDENT_IT, WORKING_STUDENT_RT));
            out.personal_diff += DIFF_WORKING_STUDENT;
        }

        out
    }
}

// ───────────── 保険料控除 ─────────────

fn life_new_it(x: i64, cap: i64) -> i64 {
    // cap=40,000（通常）/ 60,000（令和8年分 23歳未満扶養親族の特例）
    if x <= 0 {
        0
    } else if x <= cap / 2 {
        x
    } else if x <= cap {
        ceil_div(x, 2) + cap / 4
    } else if x <= cap * 2 {
        ceil_div(x, 4) + cap / 2
    } else {
        cap
    }
}

/// 生命保険料控除の算式は1円未満切上げ
fn ceil_div(x: i64, d: i64) -> i64 {
    (x + d - 1) / d
}

fn life_old_it(x: i64) -> i64 {
    if x <= 0 {
        0
    } else if x <= 25_000 {
        x
    } else if x <= 50_000 {
        ceil_div(x, 2) + 12_500
    } else if x <= 100_000 {
        ceil_div(x, 4) + 25_000
    } else {
        50_000
    }
}

fn life_new_rt(x: i64) -> i64 {
    if x <= 0 {
        0
    } else if x <= 12_000 {
        x
    } else if x <= 32_000 {
        ceil_div(x, 2) + 6_000
    } else if x <= 56_000 {
        ceil_div(x, 4) + 14_000
    } else {
        28_000
    }
}

fn life_old_rt(x: i64) -> i64 {
    if x <= 0 {
        0
    } else if x <= 15_000 {
        x
    } else if x <= 40_000 {
        ceil_div(x, 2) + 7_500
    } else if x <= 70_000 {
        ceil_div(x, 4) + 17_500
    } else {
        35_000
    }
}

/// 新旧両方ある区分：旧のみ適用額と（新＋旧, 上限）の大きい方
fn combine(new_amt: i64, old_amt: i64, old_raw: i64, combined_cap: i64) -> i64 {
    if old_raw > 0 && new_amt > 0 {
        old_amt.max((new_amt + old_amt).min(combined_cap))
    } else {
        new_amt + old_amt
    }
}

pub fn life_insurance(ins: &InsurancePremiumInput, kind: Kind, has_dependent_under_23: bool) -> i64 {
    // ceil 処理：法令上は1円未満切上げ
    match kind {
        Kind::IncomeTax => {
            let general_cap = if has_dependent_under_23 { LIFE_GENERAL_CAP_SPECIAL_IT } else { 40_000 };
            let general = combine(
                life_new_it(ins.life_new, general_cap),
                life_old_it(ins.life_old),
                ins.life_old,
                general_cap,
            );
            let care = life_new_it(ins.medical_care, 40_000);
            let pension = combine(
                life_new_it(ins.pension_new, 40_000),
                life_old_it(ins.pension_old),
                ins.pension_old,
                40_000,
            );
            (general + care + pension).min(120_000)
        }
        Kind::Resident => {
            let general = combine(life_new_rt(ins.life_new), life_old_rt(ins.life_old), ins.life_old, 28_000);
            let care = life_new_rt(ins.medical_care);
            let pension = combine(life_new_rt(ins.pension_new), life_old_rt(ins.pension_old), ins.pension_old, 28_000);
            (general + care + pension).min(70_000)
        }
    }
}

pub fn earthquake_insurance(ins: &InsurancePremiumInput, kind: Kind) -> i64 {
    let eq = ins.earthquake.max(0);
    let old = ins.old_long_term.max(0);
    match kind {
        Kind::IncomeTax => {
            let a = eq.min(50_000);
            let b = if old <= 10_000 { old } else if old <= 20_000 { old / 2 + 5_000 } else { 15_000 };
            (a + b).min(50_000)
        }
        Kind::Resident => {
            let a = (eq / 2).min(25_000);
            let b = if old <= 5_000 { old } else if old <= 15_000 { old / 2 + 2_500 } else { 10_000 };
            (a + b).min(25_000)
        }
    }
}

/// 医療費控除（`total_income_base` は総所得金額等）
pub fn medical(m: &MedicalInput, total_income_base: i64) -> i64 {
    match m.mode {
        MedicalMode::Normal => {
            let net = (m.expenses - m.reimbursed).max(0);
            let threshold = (total_income_base * 5 / 100).min(100_000).max(0);
            (net - threshold).clamp(0, 2_000_000)
        }
        MedicalMode::SelfMedication => (m.otc_purchases - 12_000).clamp(0, 88_000),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn life_it() {
        assert_eq!(life_new_it(20_000, 40_000), 20_000);
        assert_eq!(life_new_it(40_000, 40_000), 30_000);
        assert_eq!(life_new_it(80_000, 40_000), 40_000);
        assert_eq!(life_new_it(100_000, 40_000), 40_000);
        // 特例上限6万：3万以下全額、6万以下 x/2+1.5万、12万以下 x/4+3万
        assert_eq!(life_new_it(30_000, 60_000), 30_000);
        assert_eq!(life_new_it(60_000, 60_000), 45_000);
        assert_eq!(life_new_it(120_000, 60_000), 60_000);
        assert_eq!(life_new_it(200_000, 60_000), 60_000);
    }

    #[test]
    fn life_rt() {
        assert_eq!(life_new_rt(56_000), 28_000);
        assert_eq!(life_old_rt(70_000), 35_000);
        let ins = InsurancePremiumInput { life_new: 100_000, medical_care: 100_000, pension_new: 100_000, ..Default::default() };
        assert_eq!(life_insurance(&ins, Kind::Resident, false), 70_000);
        assert_eq!(life_insurance(&ins, Kind::IncomeTax, false), 120_000);
    }
}

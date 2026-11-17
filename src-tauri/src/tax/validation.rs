//! 入力の矛盾や、概算に含めていないものを知らせる警告

use super::deductions::relative_income;
use super::fmt_yen;
use super::model::*;
use super::params::*;

/// 入力内容から警告を作る（`total_income` は所得税の合計所得金額）
pub fn input_warnings(input: &TaxInput, total_income: i64) -> Vec<String> {
    let mut w = Vec::new();
    let fam = &input.family;
    let single_parent = matches!(fam.widow_status, WidowStatus::SingleMother | WidowStatus::SingleFather);

    if fam.widow_status != WidowStatus::None {
        if fam.has_spouse {
            w.push("配偶者がいる場合、寡婦控除・ひとり親控除は受けられません。入力を確認してください。".into());
        }
        if total_income > WIDOW_INCOME_LIMIT {
            w.push(format!(
                "合計所得金額が{}円を超えるため、寡婦控除・ひとり親控除は適用していません。",
                fmt_yen(WIDOW_INCOME_LIMIT)
            ));
        }
    }
    let has_qualifying_relative = fam
        .dependents
        .iter()
        .any(|d| relative_income(d.salary_income, d.other_income) <= DEPENDENT_INCOME_LIMIT);
    if single_parent && !has_qualifying_relative {
        w.push("ひとり親控除は、生計を一にする子（所得要件を満たす人）がいる場合に限られます。扶養親族の欄に子を入力してください。".into());
    }
    if fam.working_student && total_income > WORKING_STUDENT_INCOME_LIMIT {
        w.push(format!(
            "合計所得金額が{}円を超えるため、勤労学生控除は適用していません。",
            fmt_yen(WORKING_STUDENT_INCOME_LIMIT)
        ));
    }

    let si = &input.social_insurance;
    if si.mode == SocialInsuranceMode::Auto && si.health_plan == HealthPlan::Custom && si.custom_health_rate_x1000 <= 0 {
        w.push("健康保険料率が入力されていないため、健康保険料を0円として概算しています。".into());
    }
    if input.social_insurance.mode == SocialInsuranceMode::Auto {
        if input.age >= HEALTH_AGE_UNTIL {
            w.push("75歳以上は健康保険・厚生年金の対象外として概算しています。後期高齢者医療制度の保険料は「その他に支払った社会保険料」に入力してください。".into());
        } else if input.age >= PENSION_AGE_UNTIL {
            w.push("70歳以上は厚生年金保険料がかからないものとして概算しています。".into());
        }
        if input.age >= CARE_AGE_UNTIL {
            w.push("65歳以上の介護保険料（第1号被保険者）は概算に含めていません。「その他に支払った社会保険料」に入力してください。".into());
        }
    }
    w
}

#[cfg(test)]
mod tests {
    use super::*;

    fn has(w: &[String], part: &str) -> bool {
        w.iter().any(|s| s.contains(part))
    }

    #[test]
    fn no_warnings_by_default() {
        let input = TaxInput { age: 35, ..Default::default() };
        assert!(input_warnings(&input, 3_000_000).is_empty());
    }

    #[test]
    fn widow_and_single_parent() {
        let mut input = TaxInput { age: 40, ..Default::default() };
        input.family.widow_status = WidowStatus::SingleMother;
        // 子の入力がない
        assert!(has(&input_warnings(&input, 3_000_000), "生計を一にする子"));
        input.family.dependents = vec![DependentInput { age: 10, ..Default::default() }];
        assert!(input_warnings(&input, 3_000_000).is_empty());
        // 配偶者あり
        input.family.has_spouse = true;
        assert!(has(&input_warnings(&input, 3_000_000), "配偶者がいる場合"));
        input.family.has_spouse = false;
        // 合計所得金額500万円の境界
        assert!(input_warnings(&input, 5_000_000).is_empty());
        assert!(has(&input_warnings(&input, 5_000_001), "5,000,000円を超える"));
    }

    #[test]
    fn working_student_income_limit() {
        let mut input = TaxInput { age: 20, ..Default::default() };
        input.family.working_student = true;
        assert!(input_warnings(&input, 890_000).is_empty());
        assert!(has(&input_warnings(&input, 890_001), "勤労学生控除は適用していません"));
    }

    #[test]
    fn age_notes_only_for_estimate() {
        let mut input = TaxInput { age: 64, ..Default::default() };
        assert!(input_warnings(&input, 0).is_empty());
        input.age = 65;
        assert!(has(&input_warnings(&input, 0), "第1号被保険者"));
        input.age = 70;
        assert!(has(&input_warnings(&input, 0), "70歳以上"));
        input.age = 75;
        assert!(has(&input_warnings(&input, 0), "後期高齢者医療制度"));
        input.social_insurance.mode = SocialInsuranceMode::Manual;
        assert!(input_warnings(&input, 0).is_empty());
    }
}

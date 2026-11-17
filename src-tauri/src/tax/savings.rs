//! 控除ごとの節税額（その控除がなかった場合の税額との差）

use super::compute;
use super::model::*;

/// 比べる控除：名前と、その控除の入力を取り除く処理
struct Target {
    label: &'static str,
    remove: fn(&mut TaxInput),
}

static TARGETS: &[Target] = &[
    Target {
        label: "iDeCo・小規模企業共済等", remove: |i| i.pension_savings = PensionSavingsInput::default()
    },
    Target {
        label: "生命保険料控除",
        remove: |i| {
            i.insurance = InsurancePremiumInput {
                earthquake: i.insurance.earthquake,
                old_long_term: i.insurance.old_long_term,
                ..Default::default()
            }
        },
    },
    Target {
        label: "地震保険料控除",
        remove: |i| {
            i.insurance.earthquake = 0;
            i.insurance.old_long_term = 0;
        },
    },
    Target { label: "医療費控除", remove: |i| i.medical = MedicalInput::default() },
    Target { label: "住宅ローン控除", remove: |i| i.housing_loan = HousingLoanInput::default() },
    Target {
        label: "その他の寄附金",
        remove: |i| {
            i.donations.other_income_tax_only = 0;
            i.donations.other_resident_designated = 0;
        },
    },
];

/// 控除ごとの節税額。効果のない控除（入力がない、足切り以下など）は含めない
///
/// `current` は現在の入力での計算結果。ふるさと納税は `FurusatoResult` で別に示すので対象外。
/// 1つずつ取り除いて比べるので、各行の合計は全部を同時にやめた場合の差とは一致しない。
pub fn deduction_savings(
    input: &TaxInput,
    furusato_total: i64,
    one_stop: bool,
    current: &TaxResult,
) -> Vec<SavingItem> {
    TARGETS
        .iter()
        .filter_map(|t| {
            let mut without = input.clone();
            (t.remove)(&mut without);
            let r = compute(&without, furusato_total, one_stop).result;
            let income_tax = r.income_tax.total - current.income_tax.total;
            let resident_tax = r.resident_tax.total - current.resident_tax.total;
            (income_tax != 0 || resident_tax != 0).then(|| SavingItem {
                label: t.label.into(),
                income_tax,
                resident_tax,
                total: income_tax + resident_tax,
            })
        })
        .collect()
}

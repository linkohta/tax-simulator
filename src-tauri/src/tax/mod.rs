//! 令和8年分所得税・令和9年度住民税の計算エンジン

pub mod comparison;
pub mod curve;
pub mod deductions;
pub mod foreign_tax;
pub mod housing_loan;
pub mod misc_income;
pub mod model;
pub mod monthly;
pub mod params;
pub mod savings;
pub mod social;
pub mod stocks;
pub mod validation;

#[cfg(test)]
mod tests;

use deductions::{FamilyContext, Kind};
use model::*;
use params::*;

fn floor_to(x: i64, unit: i64) -> i64 {
    if x <= 0 {
        0
    } else {
        x / unit * unit
    }
}

/// 所得税の速算表
pub fn progressive_income_tax(taxable: i64) -> i64 {
    let row = INCOME_TAX_BRACKETS.iter().find(|b| taxable <= b.upper).unwrap_or(INCOME_TAX_BRACKETS.last().unwrap());
    (taxable * row.rate_pct / 100 - row.deduction).max(0)
}

fn marginal_bracket(taxable: i64) -> &'static Bracket {
    INCOME_TAX_BRACKETS.iter().find(|b| taxable <= b.upper).unwrap_or(INCOME_TAX_BRACKETS.last().unwrap())
}

/// 計算の途中結果（ふるさと納税上限の探索で何度も呼ぶため、入力と寄附額を分離）
struct Computed {
    result: TaxResult,
    /// ワンストップ特例が実際に適用されたか
    one_stop: bool,
}

/// 金額×(分子/分母) を切捨て
fn mul_div(x: i64, num: i64, den: i64) -> i64 {
    (x as i128 * num as i128 / den as i128) as i64
}

fn compute(input: &TaxInput, furusato_total: i64, one_stop_requested: bool) -> Computed {
    let mut warnings = Vec::new();
    let fam = &input.family;

    // ── 社会保険料 ──
    let si = social::estimate(&input.social_insurance, input.age);

    // ── 給与所得 ──
    let salary = input.salary.annual_income.max(0);
    let salary_ded_it = deductions::salary_deduction(salary, Kind::IncomeTax);
    let salary_ded_rt = deductions::salary_deduction(salary, Kind::Resident);

    let adj = deductions::income_adjustment(salary, deductions::income_adjustment_eligible(fam));
    let salary_before_pension_it = (salary - salary_ded_it - adj).max(0);
    let salary_before_pension_rt = (salary - salary_ded_rt - adj).max(0);

    // ── 株式 ──
    let st = &input.stocks;
    let stock = stocks::stock_income(st);
    warnings.extend(stock.warnings.iter().cloned());
    let dividends = stock.dividends_total;
    let div_comp = stock.div_comp.total();
    let (div_sep, gain, loss_next) = (stock.div_sep, stock.gain, stock.loss_next);

    // ── 雑所得（公的年金等・副業）。給与と年金の両方があれば、給与所得から所得金額調整控除を引く ──
    let stock_income = div_comp + stock.separate_before_carry;
    let misc = misc_income::misc_income(&input.misc_income, input.age, salary_before_pension_it, stock_income);
    let misc_rt = misc_income::misc_income(&input.misc_income, input.age, salary_before_pension_rt, stock_income);
    let salary_net_it = salary_before_pension_it - misc.salary_adjustment;
    let salary_net_rt = salary_before_pension_rt - misc_rt.salary_adjustment;
    let (misc_reason, misc_notes) = misc_income::filing(&input.misc_income, salary, &misc);
    warnings.extend(misc_notes);

    let total_income_it = salary_net_it + misc.total + stock_income; // 合計所得金額（繰越控除前）
    let total_income_rt = salary_net_rt + misc_rt.total + stock_income;
    let separate_income = div_sep + gain;
    let ordinary_it = salary_net_it + misc.total + div_comp;
    let ordinary_rt = salary_net_rt + misc_rt.total + div_comp;
    let total_base_it = ordinary_it + separate_income; // 総所得金額等
    let total_base_rt = ordinary_rt + separate_income;

    // 申告不要とした株式の源泉徴収と、その所得（ミニマム税の目安判定用）
    let (wh_it, wh_rt, wh_base) = (stock.wh_it, stock.wh_rt, stock.wh_base);

    // ── 確定申告の要否とワンストップ特例 ──
    let furusato_count = input.donations.furusato.iter().filter(|e| e.amount > 0).count();
    let other_donations =
        input.donations.other_income_tax_only.max(0) + input.donations.other_resident_designated.max(0);
    let medical_claimed = deductions::medical(&input.medical, total_base_it) > 0;
    let mut filing_reasons: Vec<&str> = Vec::new();
    if medical_claimed {
        filing_reasons.push("医療費控除");
    }
    if st.dividend_mode != DividendMode::NoDeclare && dividends > 0 {
        filing_reasons.push("配当の申告");
    }
    if stock.declares_gain {
        filing_reasons.push("株式譲渡の申告");
    }
    if st.foreign_tax_paid > 0 && st.dividend_mode != DividendMode::NoDeclare {
        filing_reasons.push("外国税額控除");
    }
    if other_donations > 0 {
        filing_reasons.push("その他の寄附金控除");
    }
    if salary > SALARY_FILING_REQUIRED_OVER {
        filing_reasons.push("給与収入2,000万円超");
    }
    filing_reasons.extend(misc_reason);
    let mut one_stop = one_stop_requested && furusato_total > 0;
    if one_stop && !filing_reasons.is_empty() {
        one_stop = false;
        warnings.push(format!(
            "確定申告が必要なため（{}）ワンストップ特例は無効になります。ふるさと納税も確定申告で寄附金控除を受けてください。",
            filing_reasons.join("・")
        ));
    }
    if one_stop && furusato_count > ONE_STOP_MAX_DONEES {
        one_stop = false;
        warnings.push("ワンストップ特例は寄附先5団体以内に限られます。確定申告として計算しました。".into());
    }
    let filing_required = !filing_reasons.is_empty() || (furusato_total > 0 && !one_stop);

    // ── 所得控除 ──
    let ctx_it = FamilyContext { family: fam, total_income: total_income_it };
    let ctx_rt = FamilyContext { family: fam, total_income: total_income_rt };
    let personal_it = ctx_it.personal(Kind::IncomeTax);
    let personal_rt = ctx_rt.personal(Kind::Resident);

    let ps = &input.pension_savings;
    let small_biz = ps.ideco.max(0) + ps.small_business_mutual_aid.max(0) + ps.corporate_dc_matching.max(0);
    let common = |kind: Kind, total_base: i64| {
        let mut v = vec![
            DeductionItem { label: "社会保険料控除".into(), amount: si.total },
            DeductionItem { label: "小規模企業共済等掛金控除（iDeCo等）".into(), amount: small_biz },
            DeductionItem {
                label: "生命保険料控除".into(),
                amount: deductions::life_insurance(&input.insurance, kind, personal_it.has_dependent_under_23),
            },
            DeductionItem {
                label: "地震保険料控除".into(),
                amount: deductions::earthquake_insurance(&input.insurance, kind),
            },
            DeductionItem {
                label: if input.medical.mode == MedicalMode::SelfMedication {
                    "医療費控除（セルフメディケーション）".into()
                } else {
                    "医療費控除".into()
                },
                amount: deductions::medical(&input.medical, total_base),
            },
        ];
        if kind == Kind::IncomeTax {
            let donations = if one_stop { 0 } else { furusato_total.max(0) } + other_donations;
            let ded = (donations.min(total_base * DONATION_IT_INCOME_LIMIT_PCT / 100) - DONATION_SELF_BURDEN).max(0);
            v.push(DeductionItem { label: "寄附金控除".into(), amount: ded });
        }
        v.retain(|d| d.amount > 0);
        v
    };
    let mut ded_it = common(Kind::IncomeTax, total_base_it);
    ded_it.extend(personal_it.items.iter().cloned());
    let mut ded_rt = common(Kind::Resident, total_base_rt);
    ded_rt.extend(personal_rt.items.iter().cloned());
    let total_ded_it: i64 = ded_it.iter().map(|d| d.amount).sum();
    let total_ded_rt: i64 = ded_rt.iter().map(|d| d.amount).sum();

    // ── 所得税 ──
    let taxable_ord_it = floor_to(ordinary_it - total_ded_it, TAXABLE_INCOME_UNIT);
    let leftover_it = (total_ded_it - ordinary_it).max(0);
    let taxable_sep_it = floor_to(separate_income - leftover_it, TAXABLE_INCOME_UNIT);
    let tax_ord = progressive_income_tax(taxable_ord_it);
    let tax_sep = taxable_sep_it * STOCK_SEPARATE_IT_RATE_PCT / 100;
    let mut credits_it = Vec::new();
    let mut tax_it = tax_ord + tax_sep;

    // 配当控除（総合課税分）
    let taxable_total_it = taxable_ord_it + taxable_sep_it;
    let div_credit_it = stocks::dividend_credit(&stock.div_comp, taxable_total_it, &DIVIDEND_CREDIT_IT).min(tax_it);
    if div_credit_it > 0 {
        tax_it -= div_credit_it;
        credits_it.push(DeductionItem { label: "配当控除".into(), amount: div_credit_it });
    }
    // 住宅借入金等特別控除
    let (hl_total, hl_warnings) = housing_loan::credit(&input.housing_loan, total_income_it);
    warnings.extend(hl_warnings);
    let hl_it = hl_total.min(tax_it);
    if hl_it > 0 {
        tax_it -= hl_it;
        credits_it.push(DeductionItem { label: "住宅借入金等特別控除".into(), amount: hl_it });
    }
    let base_tax = tax_it;
    let reconstruction = base_tax * RECONSTRUCTION_RATE_X1000 / RATE_X1000_DENOM;
    // 外国税額控除。復興特別所得税は外国税額控除前の所得税額（基準所得税額）に対してかかる
    let (ftc, ftc_warnings) = foreign_tax::for_stocks(st, total_income_it, base_tax, reconstruction);
    warnings.extend(ftc_warnings);
    let ftc_it = ftc.income_tax + ftc.reconstruction;
    let income_tax_total = floor_to(base_tax + reconstruction - ftc_it, TAX_UNIT);
    let marginal = marginal_bracket(taxable_ord_it);

    // ── 住民税 ──
    let taxable_ord_rt = floor_to(ordinary_rt - total_ded_rt, TAXABLE_INCOME_UNIT);
    let leftover_rt = (total_ded_rt - ordinary_rt).max(0);
    let taxable_sep_rt = floor_to(separate_income - leftover_rt, TAXABLE_INCOME_UNIT);
    // [市町村, 道府県]
    let rate =
        |i: usize| taxable_ord_rt * RESIDENT_RATE_PCT[i] / 100 + taxable_sep_rt * STOCK_SEPARATE_RT_RATE_PCT[i] / 100;
    let mut levy = [rate(0), rate(1)];
    let levy_before = levy[0] + levy[1];
    let mut credits_rt = Vec::new();
    let apply = |levy: &mut [i64; 2], label: &str, amounts: [i64; 2], credits: &mut Vec<DeductionItem>| {
        let a = [amounts[0].min(levy[0]).max(0), amounts[1].min(levy[1]).max(0)];
        levy[0] -= a[0];
        levy[1] -= a[1];
        if a[0] + a[1] > 0 {
            credits.push(DeductionItem { label: label.into(), amount: a[0] + a[1] });
        }
    };

    // 調整控除
    let personal_diff = personal_rt.personal_diff;
    // 所得税の基礎控除額のうち48万円を超える部分（ふるさと納税・住宅ローン控除の調整用）
    let basic_excess =
        (deductions::basic_deduction(total_income_it, Kind::IncomeTax) - BASIC_DEDUCTION_EXCESS_BASE).max(0);
    if let Some(amounts) = adjustment_credit(taxable_ord_rt, personal_diff, total_income_rt) {
        apply(&mut levy, "調整控除", amounts, &mut credits_rt);
    }
    let levy_after_adjust = levy[0] + levy[1];

    // 配当控除
    if div_comp > 0 {
        let taxable_total_rt = taxable_ord_rt + taxable_sep_rt;
        let m = stocks::dividend_credit(&stock.div_comp, taxable_total_rt, &DIVIDEND_CREDIT_RT_MUNI);
        let p = stocks::dividend_credit(&stock.div_comp, taxable_total_rt, &DIVIDEND_CREDIT_RT_PREF);
        apply(&mut levy, "配当控除", [m, p], &mut credits_rt);
    }

    // 住宅借入金等特別控除（所得税で引ききれなかった分）
    let hl_rest = hl_total - hl_it;
    if hl_rest > 0 {
        let cap = housing_loan::resident_limit(&input.housing_loan, taxable_total_it, basic_excess);
        let amt = hl_rest.min(cap);
        apply(&mut levy, "住宅借入金等特別控除", split_resident(amt), &mut credits_rt);
    }

    // 寄附金税額控除
    let furusato = furusato_total.max(0);
    let designated = input.donations.other_resident_designated.max(0);
    if furusato + designated > 0 {
        let basic_base = ((furusato + designated).min(total_base_rt * DONATION_RT_INCOME_LIMIT_PCT / 100)
            - DONATION_SELF_BURDEN)
            .max(0);
        let basic = [basic_base * RESIDENT_RATE_PCT[0] / 100, basic_base * RESIDENT_RATE_PCT[1] / 100];
        apply(&mut levy, "寄附金税額控除（基本分）", basic, &mut credits_rt);

        if furusato > DONATION_SELF_BURDEN {
            // 人的控除差調整額 = 調整控除の人的控除差 + 所得税の基礎控除の48万円超過分
            let band = furusato_band(taxable_ord_rt, taxable_sep_rt, personal_diff + basic_excess);
            let special_total = mul_div(furusato - DONATION_SELF_BURDEN, band.special_x1000, RATE_X1000_DENOM);
            let cap = levy_after_adjust * FURUSATO_SPECIAL_CAP_PCT / 100;
            let special = special_total.min(cap);
            apply(&mut levy, "寄附金税額控除（特例分）", split_resident(special), &mut credits_rt);
            if one_stop {
                let filing_sp = mul_div(special, band.filing_ratio.0, band.filing_ratio.1);
                apply(&mut levy, "寄附金税額控除（申告特例分）", split_resident(filing_sp), &mut credits_rt);
            }
        }
    }
    // 外国税額控除（所得税・復興特別所得税で引ききれなかった分）
    apply(&mut levy, "外国税額控除", ftc.resident, &mut credits_rt);
    levy = [floor_to(levy[0], TAX_UNIT), floor_to(levy[1], TAX_UNIT)];

    // 非課税判定（標準：1級地）
    let rt_count = personal_rt.household_count;
    let rt_has_deps = personal_rt.has_dependents_for_exemption;
    let protected =
        (matches!(fam.self_disability, Disability::General | Disability::Special | Disability::SpecialCohabiting)
            || fam.widow_status != WidowStatus::None
            || input.age < RT_MINOR_AGE_UNTIL)
            && total_income_rt <= RT_PROTECTED_INCOME_LIMIT;
    let per_capita_exempt = protected || total_income_rt <= per_capita_exempt_limit(rt_count, rt_has_deps);
    let levy_exempt = per_capita_exempt || total_base_rt <= income_levy_exempt_limit(rt_count, rt_has_deps);
    let income_levy = if levy_exempt { 0 } else { levy[0] + levy[1] };
    let per_capita = if per_capita_exempt { 0 } else { PER_CAPITA_TOTAL };
    let resident_total = income_levy + per_capita;

    // ── 警告 ──
    if total_income_it + wh_base > MINIMUM_TAX_THRESHOLD {
        warnings.push(
            "基準所得金額3.3億円超の場合の「極めて高い水準の所得に対する負担の適正化措置」は考慮していません。".into(),
        );
    }
    if st.dividend_mode != DividendMode::NoDeclare && dividends > 0 {
        warnings.push("配当を申告すると合計所得金額に含まれ、住民税や国保・医療費の負担区分、扶養判定に影響します（所得税と住民税は同じ課税方式）。".into());
    }

    let stock_wh = wh_it + wh_rt;
    let misc_in = &input.misc_income;
    let gross =
        salary + misc_in.pension_income.max(0) + misc_in.side_revenue.max(0) + dividends + stock.net_gain.max(0);
    let total_tax = income_tax_total + resident_total + stock_wh;
    let take_home =
        salary + misc_in.pension_income.max(0) + misc.side_net + dividends + stock.net_gain - si.total - total_tax;
    let effective = if gross > 0 { total_tax as f64 / gross as f64 * 100.0 } else { 0.0 };

    let resident_monthly = monthly::resident_special_collection(resident_total);
    let result = TaxResult {
        income: IncomeBreakdown {
            salary_income: salary,
            salary_deduction: salary_ded_it,
            income_adjustment_deduction: adj + misc.salary_adjustment,
            salary_net: salary_net_it,
            pension_net: misc.pension_net,
            side_net: misc.side_net,
            misc_income: misc.total,
            dividend_comprehensive: div_comp,
            dividend_separate: div_sep,
            capital_gain_separate: gain,
            total_income: total_base_it,
            total_income_before_carryover: total_income_it,
            loss_carried_to_next_year: loss_next,
        },
        social_insurance: si.clone(),
        income_tax: TaxDetail {
            deductions: ded_it,
            total_deductions: total_ded_it,
            taxable_ordinary: taxable_ord_it,
            taxable_separate: taxable_sep_it,
            tax_on_ordinary: tax_ord,
            tax_on_separate: tax_sep,
            credits: credits_it,
            base_tax,
            reconstruction_tax: reconstruction,
            foreign_tax_credit: ftc_it,
            total: income_tax_total,
            marginal_rate: marginal.rate_pct as f64,
        },
        resident_tax: ResidentTaxDetail {
            deductions: ded_rt,
            total_deductions: total_ded_rt,
            taxable_ordinary: taxable_ord_rt,
            taxable_separate: taxable_sep_rt,
            income_levy_before_credits: levy_before,
            credits: credits_rt,
            income_levy,
            per_capita,
            total: resident_total,
            income_levy_exempt: levy_exempt,
            per_capita_exempt,
            monthly_first: resident_monthly.0,
            monthly_rest: resident_monthly.1,
        },
        stock_withholding: StockWithholding { income_tax: wh_it, resident_tax: wh_rt },
        furusato: FurusatoResult::default(),
        stock_options: Vec::new(),
        savings: Vec::new(),
        summary: Summary {
            gross_income: gross,
            social_insurance: si.total,
            income_tax: income_tax_total,
            resident_tax: resident_total,
            stock_withholding: stock_wh,
            total_tax,
            take_home,
            take_home_monthly: monthly::monthly_average(take_home),
            effective_tax_rate: (effective * 100.0).round() / 100.0,
            filing_required,
        },
        warnings,
    };
    Computed { result, one_stop }
}

/// 調整控除 [市町村, 道府県]（地方税法37条・314条の6）。適用がなければ `None`
fn adjustment_credit(taxable_ord: i64, personal_diff: i64, total_income: i64) -> Option<[i64; 2]> {
    if total_income > ADJUSTMENT_CREDIT_INCOME_LIMIT || taxable_ord <= 0 {
        return None;
    }
    let rate = ADJUSTMENT_CREDIT_RATE_PCT;
    let min = ADJUSTMENT_CREDIT_MIN;
    if taxable_ord <= ADJUSTMENT_CREDIT_THRESHOLD {
        // 人的控除差と合計課税所得金額の少ない方 × 5%
        let base = personal_diff.min(taxable_ord);
        return Some([base * rate[0] / 100, base * rate[1] / 100]);
    }
    // {人的控除差 −（合計課税所得金額 − 200万円）} × 5%。2,500円未満なら2,500円
    let base = personal_diff - (taxable_ord - ADJUSTMENT_CREDIT_THRESHOLD);
    if base * (rate[0] + rate[1]) / 100 < min[0] + min[1] {
        Some(min)
    } else {
        Some([base * rate[0] / 100, base * rate[1] / 100])
    }
}

/// 住民税の税額控除を標準税率の比（市6：県4）で分ける。端数は道府県側に寄せる
fn split_resident(amount: i64) -> [i64; 2] {
    let muni = amount * RESIDENT_RATE_PCT[0] / (RESIDENT_RATE_PCT[0] + RESIDENT_RATE_PCT[1]);
    [muni, amount - muni]
}

/// ふるさと納税の特例控除割合の区分を決める
fn furusato_band(taxable_ord: i64, taxable_sep: i64, personal_diff: i64) -> &'static FurusatoBand {
    if taxable_ord > 0 {
        let x = taxable_ord - personal_diff;
        if x < 0 {
            return &FURUSATO_BAND_NEGATIVE;
        }
        FURUSATO_BANDS.iter().find(|b| x <= b.upper).unwrap_or(FURUSATO_BANDS.last().unwrap())
    } else if taxable_sep > 0 {
        &FURUSATO_BAND_SEPARATE
    } else {
        &FURUSATO_BAND_NEGATIVE
    }
}

fn fmt_yen(n: i64) -> String {
    let s = n.abs().to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    if n < 0 {
        format!("-{out}")
    } else {
        out
    }
}

/// 公開API：入力から計算結果（ふるさと納税上限の目安を含む）を返す
pub fn calculate(input: &TaxInput) -> TaxResult {
    let input = &input.normalized();
    let furusato_total: i64 = input.donations.furusato.iter().map(|e| e.amount.max(0)).sum();
    let one_stop = input.donations.one_stop;
    let current = compute(input, furusato_total, one_stop);
    let mut result = current.result;
    result.warnings.extend(validation::input_warnings(input, result.income.total_income_before_carryover));

    // 寄附なしの税額と比較
    let base = compute(input, 0, one_stop).result;
    let tax_of = |r: &TaxResult| (r.income_tax.total, r.resident_tax.total);
    let (it0, rt0) = tax_of(&base);
    let (it1, rt1) = tax_of(&result);
    let it_red = it0 - it1;
    let rt_red = rt0 - rt1;

    // 上限探索：自己負担が2,000円以下に収まる最大の寄附額（1,000円単位）
    // ワンストップ特例の可否は寄附額に依存しないため、正の寄附額で一度判定する
    let effective_one_stop =
        if furusato_total > 0 { current.one_stop } else { compute(input, FURUSATO_SEARCH_STEP, one_stop).one_stop };
    let burden = |f: i64| {
        let r = compute(input, f, effective_one_stop).result;
        f - ((it0 - r.income_tax.total) + (rt0 - r.resident_tax.total))
    };
    let step = FURUSATO_SEARCH_STEP;
    let mut lo = 0i64; // burden(lo * step) が自己負担額以下、を満たす
    let mut hi = (base.income.total_income.max(0) / step) + 1; // 寄附上限は総所得を超えない
    if burden(DONATION_SELF_BURDEN) > DONATION_SELF_BURDEN {
        hi = 0;
    }
    while lo < hi {
        let mid = (lo + hi + 1) / 2;
        if burden(mid * step) <= DONATION_SELF_BURDEN {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    let limit = if lo * step >= DONATION_SELF_BURDEN { lo * step } else { 0 };
    if furusato_total > 0 && limit > 0 && furusato_total > limit {
        result.warnings.push(format!("ふるさと納税の合計が上限の目安（{}円）を超えています。", fmt_yen(limit)));
    }

    result.stock_options = comparison::stock_options(input, furusato_total, one_stop, result.summary.total_tax);
    result.savings = savings::deduction_savings(input, furusato_total, one_stop, &result);
    result.furusato = FurusatoResult {
        current_total: furusato_total,
        limit,
        current_burden: if furusato_total > 0 { furusato_total - it_red - rt_red } else { 0 },
        income_tax_reduction: it_red,
        resident_tax_reduction: rt_red,
    };
    result
}

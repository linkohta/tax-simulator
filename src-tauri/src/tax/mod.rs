//! 令和8年分所得税・令和9年度住民税の計算エンジン

pub mod deductions;
pub mod model;
pub mod params;
pub mod social;

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
    let row = INCOME_TAX_BRACKETS
        .iter()
        .find(|b| taxable <= b.upper)
        .unwrap_or(INCOME_TAX_BRACKETS.last().unwrap());
    (taxable * row.rate_pct / 100 - row.deduction).max(0)
}

fn marginal_bracket(taxable: i64) -> &'static Bracket {
    INCOME_TAX_BRACKETS
        .iter()
        .find(|b| taxable <= b.upper)
        .unwrap_or(INCOME_TAX_BRACKETS.last().unwrap())
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

    // 所得金額調整控除の判定に必要な親族情報（先に所得額を使わず判定できる部分）
    let has_under23 = fam.dependents.iter().any(|d| {
        d.age < 23 && deductions::relative_income(d.salary_income, d.other_income) <= DEPENDENT_INCOME_LIMIT
    });
    let special_disabled_relative = (fam.has_spouse
        && matches!(fam.spouse.disability, Disability::Special | Disability::SpecialCohabiting)
        && deductions::relative_income(fam.spouse.salary_income, fam.spouse.other_income) <= DEPENDENT_INCOME_LIMIT)
        || fam.dependents.iter().any(|d| {
            matches!(d.disability, Disability::Special | Disability::SpecialCohabiting)
                && deductions::relative_income(d.salary_income, d.other_income) <= DEPENDENT_INCOME_LIMIT
        });
    let self_special = matches!(fam.self_disability, Disability::Special | Disability::SpecialCohabiting);
    let adj = if salary > 8_500_000 && (has_under23 || special_disabled_relative || self_special) {
        (salary.min(10_000_000) - 8_500_000) / 10
    } else {
        0
    };
    let salary_net_it = (salary - salary_ded_it - adj).max(0);
    let salary_net_rt = (salary - salary_ded_rt - adj).max(0);

    // ── 株式 ──
    let st = &input.stocks;
    let dividends = st.dividends.max(0);
    let div_comp = if st.dividend_mode == DividendMode::Comprehensive { dividends } else { 0 };
    let mut div_sep = if st.dividend_mode == DividendMode::Separate { dividends } else { 0 };
    let mut gain = if st.declare_capital_gain { st.capital_gain } else { 0 };
    let mut loss_next = 0;
    if gain < 0 {
        // 上場株式等の譲渡損失と申告分離の配当との損益通算
        let offset = (-gain).min(div_sep);
        div_sep -= offset;
        loss_next += -gain - offset;
        gain = 0;
        if st.dividend_mode != DividendMode::Separate && dividends > 0 {
            warnings.push("譲渡損失を配当と損益通算するには、配当を「申告分離課税」で申告する必要があります。".into());
        }
    }
    let total_income_it = salary_net_it + div_comp + div_sep + gain; // 合計所得金額（繰越控除前）
    let total_income_rt = salary_net_rt + div_comp + div_sep + gain;
    // 繰越損失の控除（申告が前提）
    let declares_separately = st.declare_capital_gain || st.dividend_mode == DividendMode::Separate;
    let mut carry = if declares_separately { st.loss_carryforward.max(0) } else { 0 };
    if st.loss_carryforward > 0 && !declares_separately {
        warnings.push("繰越損失を控除するには、譲渡損益または配当を申告分離で申告する必要があります。".into());
    }
    let use_gain = carry.min(gain);
    gain -= use_gain;
    carry -= use_gain;
    let use_div = carry.min(div_sep);
    div_sep -= use_div;
    carry -= use_div;
    if declares_separately {
        loss_next += carry;
    }
    let separate_income = div_sep + gain;
    let ordinary_it = salary_net_it + div_comp;
    let ordinary_rt = salary_net_rt + div_comp;
    let total_base_it = ordinary_it + separate_income; // 総所得金額等
    let total_base_rt = ordinary_rt + separate_income;

    // 申告不要とした株式の源泉徴収
    let mut wh_it = 0;
    let mut wh_rt = 0;
    if st.dividend_mode == DividendMode::NoDeclare {
        wh_it += mul_div(dividends, STOCK_IT_RATE_X1000, 100_000);
        wh_rt += dividends * STOCK_RT_RATE_PCT / 100;
    }
    if !st.declare_capital_gain && st.capital_gain > 0 {
        wh_it += mul_div(st.capital_gain, STOCK_IT_RATE_X1000, 100_000);
        wh_rt += st.capital_gain * STOCK_RT_RATE_PCT / 100;
    }

    // 申告不要とした株式の所得（ミニマム税の目安判定用）
    let wh_base = if st.dividend_mode == DividendMode::NoDeclare { dividends } else { 0 }
        + if !st.declare_capital_gain { st.capital_gain.max(0) } else { 0 };

    // ── 確定申告の要否とワンストップ特例 ──
    let furusato_count = input.donations.furusato.iter().filter(|e| e.amount > 0).count();
    let other_donations = input.donations.other_income_tax_only.max(0) + input.donations.other_resident_designated.max(0);
    let medical_claimed = deductions::medical(&input.medical, total_base_it) > 0;
    let mut filing_reasons: Vec<&str> = Vec::new();
    if medical_claimed {
        filing_reasons.push("医療費控除");
    }
    if st.dividend_mode != DividendMode::NoDeclare && dividends > 0 {
        filing_reasons.push("配当の申告");
    }
    if st.declare_capital_gain {
        filing_reasons.push("株式譲渡の申告");
    }
    if other_donations > 0 {
        filing_reasons.push("その他の寄附金控除");
    }
    if salary > 20_000_000 {
        filing_reasons.push("給与収入2,000万円超");
    }
    let mut one_stop = one_stop_requested && furusato_total > 0;
    if one_stop && !filing_reasons.is_empty() {
        one_stop = false;
        warnings.push(format!(
            "確定申告が必要なため（{}）ワンストップ特例は無効になります。ふるさと納税も確定申告で寄附金控除を受けてください。",
            filing_reasons.join("・")
        ));
    }
    if one_stop && furusato_count > 5 {
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
            DeductionItem { label: "地震保険料控除".into(), amount: deductions::earthquake_insurance(&input.insurance, kind) },
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
            let ded = (donations.min(total_base * 40 / 100) - 2_000).max(0);
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
    let taxable_ord_it = floor_to(ordinary_it - total_ded_it, 1000);
    let leftover_it = (total_ded_it - ordinary_it).max(0);
    let taxable_sep_it = floor_to(separate_income - leftover_it, 1000);
    let tax_ord = progressive_income_tax(taxable_ord_it);
    let tax_sep = taxable_sep_it * 15 / 100;
    let mut credits_it = Vec::new();
    let mut tax_it = tax_ord + tax_sep;

    // 配当控除（総合課税分）
    let taxable_total_it = taxable_ord_it + taxable_sep_it;
    if div_comp > 0 {
        let c = dividend_credit(div_comp, taxable_total_it, DIVIDEND_CREDIT_IT);
        let c = c.min(tax_it);
        tax_it -= c;
        credits_it.push(DeductionItem { label: "配当控除".into(), amount: c });
    }
    // 住宅借入金等特別控除
    let hl_total = input.housing_loan.credit_amount.max(0);
    let hl_it = hl_total.min(tax_it);
    if hl_it > 0 {
        tax_it -= hl_it;
        credits_it.push(DeductionItem { label: "住宅借入金等特別控除".into(), amount: hl_it });
    }
    let base_tax = tax_it;
    let reconstruction = base_tax * RECONSTRUCTION_RATE_X1000 / 100_000;
    let income_tax_total = floor_to(base_tax + reconstruction, 100);
    let marginal = marginal_bracket(taxable_ord_it);

    // ── 住民税 ──
    let taxable_ord_rt = floor_to(ordinary_rt - total_ded_rt, 1000);
    let leftover_rt = (total_ded_rt - ordinary_rt).max(0);
    let taxable_sep_rt = floor_to(separate_income - leftover_rt, 1000);
    // [市町村, 道府県]
    let mut levy = [
        taxable_ord_rt * 6 / 100 + taxable_sep_rt * 3 / 100,
        taxable_ord_rt * 4 / 100 + taxable_sep_rt * 2 / 100,
    ];
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
    let basic_excess = (deductions::basic_deduction(total_income_it, Kind::IncomeTax) - BASIC_DEDUCTION_EXCESS_BASE).max(0);
    if total_income_rt <= 25_000_000 && taxable_ord_rt > 0 {
        let base = if taxable_ord_rt <= 2_000_000 {
            Some(personal_diff.min(taxable_ord_rt))
        } else {
            let x = personal_diff - (taxable_ord_rt - 2_000_000);
            if x * 5 / 100 < 2_500 { None } else { Some(x) }
        };
        let amounts = match base {
            Some(b) => [b * 3 / 100, b * 2 / 100],
            None => [1_500, 1_000],
        };
        apply(&mut levy, "調整控除", amounts, &mut credits_rt);
    }
    let levy_after_adjust = levy[0] + levy[1];

    // 配当控除
    if div_comp > 0 {
        let taxable_total_rt = taxable_ord_rt + taxable_sep_rt;
        let m = dividend_credit(div_comp, taxable_total_rt, DIVIDEND_CREDIT_RT_MUNI);
        let p = dividend_credit(div_comp, taxable_total_rt, DIVIDEND_CREDIT_RT_PREF);
        apply(&mut levy, "配当控除", [m, p], &mut credits_rt);
    }

    // 住宅借入金等特別控除（所得税で引ききれなかった分）
    let hl_rest = hl_total - hl_it;
    if hl_rest > 0 {
        // 限度額は所得税の課税総所得金額等×5%。令和7年以前の居住分は基礎控除の48万円超過分を加算
        let mut base = taxable_ord_it + taxable_sep_it;
        if !input.housing_loan.moved_in_2026_or_later {
            base += basic_excess;
        }
        let cap = (base * HOUSING_LOAN_RT_RATE_PCT / 100).min(HOUSING_LOAN_RT_CAP);
        let amt = hl_rest.min(cap);
        apply(&mut levy, "住宅借入金等特別控除", [amt * 3 / 5, amt - amt * 3 / 5], &mut credits_rt);
    }

    // 寄附金税額控除
    let furusato = furusato_total.max(0);
    let designated = input.donations.other_resident_designated.max(0);
    if furusato + designated > 0 {
        let basic_base = ((furusato + designated).min(total_base_rt * 30 / 100) - 2_000).max(0);
        let basic = [basic_base * 6 / 100, basic_base * 4 / 100];
        apply(&mut levy, "寄附金税額控除（基本分）", basic, &mut credits_rt);

        if furusato > 2_000 {
            // 人的控除差調整額 = 調整控除の人的控除差 + 所得税の基礎控除の48万円超過分
            let band = furusato_band(taxable_ord_rt, taxable_sep_rt, personal_diff + basic_excess);
            let special_total = mul_div(furusato - 2_000, band.special_x1000, 100_000);
            let cap = levy_after_adjust * 20 / 100;
            let special = special_total.min(cap);
            let sp = [special * 3 / 5, special - special * 3 / 5];
            apply(&mut levy, "寄附金税額控除（特例分）", sp, &mut credits_rt);
            if one_stop {
                let filing_sp = mul_div(special, band.filing_ratio.0, band.filing_ratio.1);
                apply(
                    &mut levy,
                    "寄附金税額控除（申告特例分）",
                    [filing_sp * 3 / 5, filing_sp - filing_sp * 3 / 5],
                    &mut credits_rt,
                );
            }
        }
    }
    levy = [floor_to(levy[0], 100), floor_to(levy[1], 100)];

    // 非課税判定（標準：1級地）
    let rt_count = personal_rt.household_count;
    let rt_has_deps = personal_rt.has_dependents_for_exemption;
    let protected = (matches!(fam.self_disability, Disability::General | Disability::Special | Disability::SpecialCohabiting)
        || fam.widow_status != WidowStatus::None
        || input.age < 18)
        && total_income_rt <= RT_PROTECTED_INCOME_LIMIT;
    let per_capita_exempt = protected || total_income_rt <= per_capita_exempt_limit(rt_count, rt_has_deps);
    let levy_exempt = per_capita_exempt || total_base_rt <= income_levy_exempt_limit(rt_count, rt_has_deps);
    let income_levy = if levy_exempt { 0 } else { levy[0] + levy[1] };
    let per_capita = if per_capita_exempt { 0 } else { PER_CAPITA_TOTAL };
    let resident_total = income_levy + per_capita;

    // ── 警告 ──
    if total_income_it + wh_base > 300_000_000 {
        warnings.push("基準所得金額3.3億円超の場合の「極めて高い水準の所得に対する負担の適正化措置」は考慮していません。".into());
    }
    if st.dividend_mode != DividendMode::NoDeclare && dividends > 0 {
        warnings.push("配当を申告すると合計所得金額に含まれ、住民税や国保・医療費の負担区分、扶養判定に影響します（所得税と住民税は同じ課税方式）。".into());
    }

    let stock_wh = wh_it + wh_rt;
    let gross = salary + dividends + st.capital_gain.max(0);
    let total_tax = income_tax_total + resident_total + stock_wh;
    let take_home = salary + dividends + st.capital_gain - si.total - total_tax;
    let effective = if gross > 0 { total_tax as f64 / gross as f64 * 100.0 } else { 0.0 };

    let result = TaxResult {
        income: IncomeBreakdown {
            salary_income: salary,
            salary_deduction: salary_ded_it,
            income_adjustment_deduction: adj,
            salary_net: salary_net_it,
            dividend_comprehensive: div_comp,
            dividend_separate: div_sep,
            capital_gain_separate: gain,
            total_income: total_base_it,
            total_income_for_tests: total_income_it,
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
        },
        stock_withholding: StockWithholding { income_tax: wh_it, resident_tax: wh_rt },
        furusato: FurusatoResult::default(),
        summary: Summary {
            gross_income: gross,
            social_insurance: si.total,
            income_tax: income_tax_total,
            resident_tax: resident_total,
            stock_withholding: stock_wh,
            total_tax,
            take_home,
            effective_tax_rate: (effective * 100.0).round() / 100.0,
            filing_required,
        },
        warnings,
    };
    Computed { result, one_stop }
}

/// 配当控除額（rate は 1/1000 %単位の [1,000万以下部分, 超過部分]）
fn dividend_credit(div: i64, taxable_total: i64, rate: [i64; 2]) -> i64 {
    let limit = 10_000_000;
    let (low, high) = if taxable_total <= limit {
        (div, 0)
    } else if taxable_total - div >= limit {
        (0, div)
    } else {
        let low = limit - (taxable_total - div);
        (low, div - low)
    };
    mul_div(low, rate[0], 100_000) + mul_div(high, rate[1], 100_000)
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
        if i > 0 && (s.len() - i) % 3 == 0 {
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
    let furusato_total: i64 = input.donations.furusato.iter().map(|e| e.amount.max(0)).sum();
    let one_stop = input.donations.one_stop;
    let current = compute(input, furusato_total, one_stop);
    let mut result = current.result;

    // 寄附なしの税額と比較
    let base = compute(input, 0, one_stop).result;
    let tax_of = |r: &TaxResult| (r.income_tax.total, r.resident_tax.total);
    let (it0, rt0) = tax_of(&base);
    let (it1, rt1) = tax_of(&result);
    let it_red = it0 - it1;
    let rt_red = rt0 - rt1;

    // 上限探索：自己負担が2,000円以下に収まる最大の寄附額（1,000円単位）
    // ワンストップ特例の可否は寄附額に依存しないため、正の寄附額で一度判定する
    let effective_one_stop = if furusato_total > 0 {
        current.one_stop
    } else {
        compute(input, 10_000, one_stop).one_stop
    };
    let burden = |f: i64| {
        let r = compute(input, f, effective_one_stop).result;
        f - ((it0 - r.income_tax.total) + (rt0 - r.resident_tax.total))
    };
    let mut lo = 0i64; // burden(lo*1000) <= 2000 を満たす
    let mut hi = (base.income.total_income.max(0) / 1000) + 1; // 寄附上限は総所得を超えない
    if burden(2_000) > 2_000 {
        hi = 0;
    }
    while lo < hi {
        let mid = (lo + hi + 1) / 2;
        if burden(mid * 1000) <= 2_000 {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    let limit = if lo * 1000 >= 2_000 { lo * 1000 } else { 0 };
    if furusato_total > 0 && limit > 0 && furusato_total > limit {
        result.warnings.push(format!(
            "ふるさと納税の合計が上限の目安（{}円）を超えています。",
            fmt_yen(limit)
        ));
    }

    result.furusato = FurusatoResult {
        current_total: furusato_total,
        limit,
        current_burden: if furusato_total > 0 { furusato_total - it_red - rt_red } else { 0 },
        income_tax_reduction: it_red,
        resident_tax_reduction: rt_red,
    };
    result
}

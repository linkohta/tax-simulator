//! 株式の課税方式の比較（配当の課税方式 × 源泉徴収ありの特定口座を申告するか）

use super::compute;
use super::model::*;
use super::stocks::{requires_dividend_declaration, DividendAmounts};

const DIVIDEND_MODES: [DividendMode; 3] = [DividendMode::NoDeclare, DividendMode::Comprehensive, DividendMode::Separate];

/// 課税方式ごとの税額。配当も、損益のある源泉徴収ありの特定口座もなければ空
///
/// `current_total_tax` は現在の入力での税額合計。
pub fn stock_options(input: &TaxInput, furusato_total: i64, one_stop: bool, current_total_tax: i64) -> Vec<StockOption> {
    let st = &input.stocks;
    let has_dividends = DividendAmounts::from_input(st).total() > 0;
    let choosable = |a: &&StockAccount| a.kind == StockAccountKind::Withholding && a.capital_gain != 0;
    let has_choosable = st.accounts.iter().any(|a| choosable(&a));
    if !has_dividends && !has_choosable {
        return Vec::new();
    }
    let modes: &[DividendMode] = if has_dividends { &DIVIDEND_MODES } else { std::slice::from_ref(&st.dividend_mode) };
    let declares: &[Option<bool>] = if has_choosable { &[Some(false), Some(true)] } else { &[None] };

    let mut options = Vec::new();
    for &mode in modes {
        for &declare in declares {
            let mut variant = input.clone();
            variant.stocks.dividend_mode = mode;
            if let Some(d) = declare {
                for a in variant.stocks.accounts.iter_mut().filter(|a| a.kind == StockAccountKind::Withholding) {
                    a.declare = d;
                }
            }
            // 法令上選べない組み合わせ（損失を申告するのに、その口座の配当を申告しない）は比べない
            if requires_dividend_declaration(&variant.stocks) {
                continue;
            }
            let r = compute(&variant, furusato_total, one_stop).result;
            let same_declare = declare.is_none_or(|d| st.accounts.iter().filter(choosable).all(|a| a.declare == d));
            options.push(StockOption {
                dividend_mode: mode,
                declare_withholding_accounts: declare,
                income_tax: r.income_tax.total,
                resident_tax: r.resident_tax.total,
                stock_withholding: r.summary.stock_withholding,
                total_tax: r.summary.total_tax,
                loss_carried_to_next_year: r.income.loss_carried_to_next_year,
                diff_from_current: r.summary.total_tax - current_total_tax,
                is_current: mode == st.dividend_mode && same_declare,
                is_best: false,
            });
        }
    }
    // 税額が同じなら、翌年に繰り越せる損失が多い方式、次に現在の方式を優先する
    let best = options
        .iter()
        .enumerate()
        .min_by_key(|(_, o)| (o.total_tax, -o.loss_carried_to_next_year, !o.is_current))
        .map(|(i, _)| i);
    if let Some(i) = best {
        options[i].is_best = true;
    }
    options
}

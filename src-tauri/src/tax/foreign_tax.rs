//! 外国税額控除（所法95条、地方税法37条の3・314条の8）

use super::fmt_yen;
use super::model::{DividendMode, StockInput};
use super::mul_div;
use super::params::*;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ForeignTaxCredit {
    /// 所得税から控除する額
    pub income_tax: i64,
    /// 復興特別所得税から控除する額
    pub reconstruction: i64,
    /// 住民税から控除する額 [市町村, 道府県]
    pub resident: [i64; 2],
    /// どの控除限度額にも収まらなかった外国所得税額
    pub unused: i64,
}

/// 外国税額控除の額
///
/// 控除限度額は「税額 × 国外所得金額 ÷ 所得総額」。所得税、復興特別所得税、道府県民税、市町村民税の順に控除する。
/// `base_tax` は配当控除・住宅借入金等特別控除を引いた後の所得税額、`total_income` は繰越控除前の所得の合計。
pub fn credit(paid: i64, foreign_income: i64, total_income: i64, base_tax: i64, reconstruction: i64) -> ForeignTaxCredit {
    let paid = paid.max(0);
    if paid == 0 || total_income <= 0 {
        return ForeignTaxCredit { unused: paid, ..Default::default() };
    }
    // 国外所得金額は所得総額を上限とする
    let foreign = foreign_income.clamp(0, total_income);
    let limit_it = mul_div(base_tax, foreign, total_income);
    let limit_rec = mul_div(reconstruction, foreign, total_income);
    let limit_pref = limit_it * FOREIGN_TAX_CREDIT_RT_PREF_PCT / 100;
    let limit_muni = limit_it * FOREIGN_TAX_CREDIT_RT_MUNI_PCT / 100;

    let mut rest = paid;
    let mut take = |limit: i64| {
        let a = rest.min(limit.max(0));
        rest -= a;
        a
    };
    let income_tax = take(limit_it);
    let reconstruction = take(limit_rec);
    let pref = take(limit_pref);
    let muni = take(limit_muni);
    ForeignTaxCredit { income_tax, reconstruction, resident: [muni, pref], unused: rest }
}

/// 株式の入力から外国税額控除を計算する。配当を申告しない場合は控除できない
pub fn for_stocks(st: &StockInput, total_income: i64, base_tax: i64, reconstruction: i64) -> (ForeignTaxCredit, Vec<String>) {
    let paid = st.foreign_tax_paid.max(0);
    if paid == 0 {
        return (ForeignTaxCredit::default(), Vec::new());
    }
    if st.dividend_mode == DividendMode::NoDeclare {
        let w = "外国税額控除を受けるには、配当を総合課税または申告分離課税で申告する必要があります。".into();
        return (ForeignTaxCredit::default(), vec![w]);
    }
    let c = credit(paid, st.foreign_income, total_income, base_tax, reconstruction);
    let mut warnings = Vec::new();
    if st.foreign_income <= 0 {
        warnings.push("外国税額控除の計算には、国外所得金額の入力が必要です。".into());
    } else if c.unused > 0 {
        warnings.push(format!(
            "控除限度額を超える外国所得税 {}円 は控除していません（翌年以後3年間の繰越控除は扱いません）。",
            fmt_yen(c.unused)
        ));
    }
    (c, warnings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limits_in_order() {
        // 所得総額 366万、国外所得 10万、所得税 103,500、復興税 2,173
        // 所得税の限度額 103,500×10/366 = 2,827、復興税 2,173×10/366 = 59
        // 道府県民税 2,827×12% = 339、市町村民税 2,827×18% = 508
        let c = credit(10_000, 100_000, 3_660_000, 103_500, 2_173);
        assert_eq!(c, ForeignTaxCredit { income_tax: 2_827, reconstruction: 59, resident: [508, 339], unused: 6_267 });
        // 所得税の限度額に収まる場合はそこで終わり
        let c = credit(2_827, 100_000, 3_660_000, 103_500, 2_173);
        assert_eq!(c, ForeignTaxCredit { income_tax: 2_827, ..Default::default() });
        // 1円超えると復興特別所得税から
        let c = credit(2_828, 100_000, 3_660_000, 103_500, 2_173);
        assert_eq!((c.income_tax, c.reconstruction, c.unused), (2_827, 1, 0));
    }

    #[test]
    fn edge_cases() {
        assert_eq!(credit(0, 100_000, 3_660_000, 103_500, 2_173), ForeignTaxCredit::default());
        // 所得がない
        assert_eq!(credit(5_000, 100_000, 0, 0, 0).unused, 5_000);
        // 国外所得金額が所得総額を超えても、限度額は税額まで
        let c = credit(500_000, 9_000_000, 3_660_000, 103_500, 2_173);
        assert_eq!((c.income_tax, c.reconstruction), (103_500, 2_173));
    }

    #[test]
    fn requires_declared_dividends() {
        let mut st = StockInput { foreign_income: 100_000, foreign_tax_paid: 10_000, ..Default::default() };
        let (c, w) = for_stocks(&st, 3_660_000, 103_500, 2_173);
        assert_eq!(c, ForeignTaxCredit::default());
        assert!(w[0].contains("申告する必要があります"));
        st.dividend_mode = DividendMode::Separate;
        let (c, w) = for_stocks(&st, 3_660_000, 103_500, 2_173);
        assert_eq!(c.income_tax, 2_827);
        assert!(w[0].contains("6,267円"));
    }
}

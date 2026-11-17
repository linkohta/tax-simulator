//! 上場株式等の配当・譲渡損益（損益通算、繰越控除、源泉徴収、配当控除）

use super::model::*;
use super::mul_div;
use super::params::*;

/// 配当等の金額（配当控除の区分ごと）
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DividendAmounts {
    /// 剰余金の配当等
    pub stock: i64,
    /// 証券投資信託
    pub fund: i64,
    /// 外貨建等証券投資信託
    pub foreign_fund: i64,
    /// 配当控除の対象外
    pub no_credit: i64,
}

impl DividendAmounts {
    pub fn from_input(st: &StockInput) -> Self {
        Self {
            stock: st.dividends.max(0),
            fund: st.dividends_fund.max(0),
            foreign_fund: st.dividends_foreign_fund.max(0),
            no_credit: st.dividends_no_credit.max(0),
        }
    }

    pub fn total(&self) -> i64 {
        self.stock + self.fund + self.foreign_fund + self.no_credit
    }
}

/// 株式の所得と源泉徴収税
#[derive(Debug, Clone, Default)]
pub struct StockIncome {
    /// 配当等の合計（課税方式によらない）
    pub dividends_total: i64,
    /// 総合課税とした配当所得
    pub div_comp: DividendAmounts,
    /// 申告分離の所得（損益通算後・繰越控除前）。合計所得金額に入れる
    pub separate_before_carry: i64,
    /// 申告分離の配当所得等（損益通算・繰越控除後）
    pub div_sep: i64,
    /// 申告分離の譲渡所得等（損益通算・繰越控除後）
    pub gain: i64,
    /// 翌年へ繰り越す譲渡損失
    pub loss_next: i64,
    /// 譲渡損益を申告する口座があるか
    pub declares_gain: bool,
    /// 全口座の譲渡損益の合計（手取りの計算用）
    pub net_gain: i64,
    /// 申告不要とした配当・譲渡益の源泉徴収税（所得税・復興税）
    pub wh_it: i64,
    /// 同（住民税）
    pub wh_rt: i64,
    /// 申告不要とした配当・譲渡益の金額
    pub wh_base: i64,
    pub warnings: Vec<String>,
}

/// その口座の譲渡損益を申告するか。源泉徴収なしの特定口座と一般口座は、損益があれば申告する
pub fn is_declared(a: &StockAccount) -> bool {
    match a.kind {
        StockAccountKind::Withholding => a.declare,
        StockAccountKind::NoWithholding | StockAccountKind::General => a.capital_gain != 0,
    }
}

/// 譲渡損失があり、配当等を受け入れている源泉徴収ありの特定口座か
fn has_loss_and_dividends(a: &StockAccount) -> bool {
    a.kind == StockAccountKind::Withholding && a.capital_gain < 0 && a.dividends_received > 0
}

/// 配当を申告不要にできない組み合わせか。
/// 源泉徴収ありの特定口座の譲渡損失を申告する場合、その口座に受け入れた配当等も申告しなければならない（措法37条の11の6第10項）
pub fn requires_dividend_declaration(st: &StockInput) -> bool {
    st.dividend_mode == DividendMode::NoDeclare && st.accounts.iter().any(|a| a.declare && has_loss_and_dividends(a))
}

/// 申告しない源泉徴収ありの特定口座の中で、譲渡損失と通算される配当等の額（措法37条の11の6第6項）。
/// `dividends` は配当等の合計で、口座に受け入れた額の入力がこれを超えていても通算はここまで
fn in_account_offset(st: &StockInput, dividends: i64) -> i64 {
    let offset: i64 = st
        .accounts
        .iter()
        .filter(|a| !a.declare && has_loss_and_dividends(a))
        .map(|a| a.dividends_received.min(-a.capital_gain))
        .sum();
    offset.min(dividends)
}

pub fn stock_income(st: &StockInput) -> StockIncome {
    let mut warnings = Vec::new();
    let amounts = DividendAmounts::from_input(st);
    let dividends = amounts.total();
    let div_comp = if st.dividend_mode == DividendMode::Comprehensive { amounts } else { DividendAmounts::default() };
    let mut div_sep = if st.dividend_mode == DividendMode::Separate { dividends } else { 0 };

    // 申告する口座の損益は口座をまたいで通算する
    let declares_gain = st.accounts.iter().any(is_declared);
    let mut gain: i64 = st.accounts.iter().filter(|a| is_declared(a)).map(|a| a.capital_gain).sum();
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
    let separate_before_carry = div_sep + gain;

    // 繰越損失の控除（申告が前提）。譲渡所得、配当所得の順に控除する
    let declares_separately = declares_gain || st.dividend_mode == DividendMode::Separate;
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
    loss_next += carry;

    // 申告不要とした分の源泉徴収。譲渡益は口座ごとに課税され、ほかの口座の損失とは通算されない
    // 配当は、申告しない口座の中で譲渡損失と通算された分を除く
    let mut wh_base =
        if st.dividend_mode == DividendMode::NoDeclare { dividends - in_account_offset(st, dividends) } else { 0 };
    let mut wh_it = mul_div(wh_base, STOCK_IT_RATE_X1000, RATE_X1000_DENOM);
    let mut wh_rt = wh_base * STOCK_RT_RATE_PCT / 100;
    for a in st.accounts.iter().filter(|a| !is_declared(a) && a.capital_gain > 0) {
        wh_base += a.capital_gain;
        wh_it += mul_div(a.capital_gain, STOCK_IT_RATE_X1000, RATE_X1000_DENOM);
        wh_rt += a.capital_gain * STOCK_RT_RATE_PCT / 100;
    }
    if st.accounts.iter().any(|a| !is_declared(a) && a.capital_gain < 0) {
        warnings.push("源泉徴収ありの特定口座の損失は、申告しないと、その口座に受け入れた配当等との通算を除いて、ほかの口座の利益や配当との損益通算、翌年への繰越ができません。".into());
    }
    if requires_dividend_declaration(st) {
        warnings.push("源泉徴収ありの特定口座の譲渡損失を申告する場合、その口座に受け入れた配当等も申告が必要です。配当の課税方式を申告分離課税か総合課税にしてください。".into());
    }
    let received: i64 = st
        .accounts
        .iter()
        .filter(|a| a.kind == StockAccountKind::Withholding)
        .map(|a| a.dividends_received.max(0))
        .sum();
    if received > dividends {
        warnings.push("口座に受け入れた配当等の合計が、配当等の合計を超えています。口座に受け入れた額は、配当等の内数として入力してください。".into());
    }

    StockIncome {
        dividends_total: dividends,
        div_comp,
        separate_before_carry,
        div_sep,
        gain,
        loss_next,
        declares_gain,
        net_gain: st.accounts.iter().map(|a| a.capital_gain).sum(),
        wh_it,
        wh_rt,
        wh_base,
        warnings,
    }
}

/// 配当控除額。`rates` は `params::DIVIDEND_CREDIT_*`（1/1000 %単位）
///
/// 課税総所得金額等が1,000万円を超える場合、剰余金の配当等は「課税総所得金額等 − 証券投資信託の配当所得」が
/// 1,000万円を超える部分に低い率を使う（所法92条1項）。つまり所得の上に 剰余金の配当等 → 証券投資信託 の順に
/// 積んで、1,000万円を超える部分を求める。証券投資信託の中では外貨建等を上に積む（簡略化）。
pub fn dividend_credit(div: &DividendAmounts, taxable_total: i64, rates: &[[i64; 2]; 3]) -> i64 {
    let pieces = [div.stock, div.fund, div.foreign_fund];
    // この区分より上に積む配当の合計
    let mut above: i64 = pieces.iter().sum();
    let mut total = 0;
    for (amount, rate) in pieces.iter().zip(rates) {
        above -= amount;
        let high = (taxable_total - above - DIVIDEND_CREDIT_THRESHOLD).clamp(0, *amount);
        total += mul_div(amount - high, rate[0], RATE_X1000_DENOM) + mul_div(high, rate[1], RATE_X1000_DENOM);
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account(kind: StockAccountKind, capital_gain: i64, declare: bool) -> StockAccount {
        StockAccount { name: String::new(), kind, capital_gain, declare, dividends_received: 0 }
    }

    #[test]
    fn dividends_are_netted_inside_undeclared_account() {
        let input =
            |capital_gain: i64, declare: bool, dividends_received: i64, dividend_mode: DividendMode| StockInput {
                dividends: 300_000,
                dividend_mode,
                accounts: vec![StockAccount {
                    dividends_received,
                    ..account(StockAccountKind::Withholding, capital_gain, declare)
                }],
                ..Default::default()
            };
        // 配当30万のうち20万を受け入れた口座に損失50万：20万が口座内で通算され、残り10万に課税
        // 10万×15.315% = 15,315、10万×5% = 5,000
        let s = stock_income(&input(-500_000, false, 200_000, DividendMode::NoDeclare));
        assert_eq!((s.wh_base, s.wh_it, s.wh_rt), (100_000, 15_315, 5_000));
        // 損失が受け入れた配当より少なければ、損失の額まで
        let s = stock_income(&input(-50_000, false, 200_000, DividendMode::NoDeclare));
        assert_eq!(s.wh_base, 250_000);
        // 口座に受け入れていなければ通算されない
        assert_eq!(stock_income(&input(-500_000, false, 0, DividendMode::NoDeclare)).wh_base, 300_000);
        // 利益の口座では通算はなく、配当と譲渡益の両方に課税
        assert_eq!(stock_income(&input(100_000, false, 200_000, DividendMode::NoDeclare)).wh_base, 400_000);
        // 受け入れた額が配当等の合計を超えていても、通算は合計まで（警告も出す）
        let s = stock_income(&input(-500_000, false, 800_000, DividendMode::NoDeclare));
        assert_eq!(s.wh_base, 0);
        assert!(s.warnings.iter().any(|w| w.contains("配当等の合計を超えています")));
        // 配当を申告する場合は口座内の通算を使わない（申告した配当に課税される）
        let s = stock_income(&input(-500_000, false, 200_000, DividendMode::Separate));
        assert_eq!((s.wh_base, s.div_sep), (0, 300_000));
    }

    #[test]
    fn declaring_loss_requires_declaring_dividends() {
        let st = |dividend_mode| StockInput {
            dividends: 300_000,
            dividend_mode,
            accounts: vec![StockAccount {
                dividends_received: 200_000,
                ..account(StockAccountKind::Withholding, -500_000, true)
            }],
            ..Default::default()
        };
        // 損失を申告するのに配当が申告不要：警告し、口座内の通算はしない
        let s = stock_income(&st(DividendMode::NoDeclare));
        assert!(requires_dividend_declaration(&st(DividendMode::NoDeclare)));
        assert!(s.warnings.iter().any(|w| w.contains("配当等も申告が必要")));
        assert_eq!(s.wh_base, 300_000);
        // 配当を申告分離にすれば問題ない
        assert!(!requires_dividend_declaration(&st(DividendMode::Separate)));
        assert_eq!(stock_income(&st(DividendMode::Separate)).loss_next, 200_000);
    }

    #[test]
    fn credit_rates_by_category() {
        let t = 5_000_000;
        let d = |stock, fund, foreign_fund, no_credit| DividendAmounts { stock, fund, foreign_fund, no_credit };
        // 所得税：10% / 5% / 2.5% / 対象外
        assert_eq!(dividend_credit(&d(200_000, 0, 0, 0), t, &DIVIDEND_CREDIT_IT), 20_000);
        assert_eq!(dividend_credit(&d(0, 200_000, 0, 0), t, &DIVIDEND_CREDIT_IT), 10_000);
        assert_eq!(dividend_credit(&d(0, 0, 200_000, 0), t, &DIVIDEND_CREDIT_IT), 5_000);
        assert_eq!(dividend_credit(&d(0, 0, 0, 200_000), t, &DIVIDEND_CREDIT_IT), 0);
        // 住民税（市町村＋道府県）：2.8% / 1.4% / 0.7%
        let rt = |x: &DividendAmounts| {
            dividend_credit(x, t, &DIVIDEND_CREDIT_RT_MUNI) + dividend_credit(x, t, &DIVIDEND_CREDIT_RT_PREF)
        };
        assert_eq!(rt(&d(200_000, 0, 0, 0)), 5_600);
        assert_eq!(rt(&d(0, 200_000, 0, 0)), 2_800);
        assert_eq!(rt(&d(0, 0, 200_000, 0)), 1_400);
    }

    #[test]
    fn credit_over_10m() {
        let stock = DividendAmounts { stock: 1_000_000, ..Default::default() };
        // 1,000万円ちょうど：全額10% = 100,000
        assert_eq!(dividend_credit(&stock, 10_000_000, &DIVIDEND_CREDIT_IT), 100_000);
        // 50万円が1,000万円超：50万×10% + 50万×5% = 75,000
        assert_eq!(dividend_credit(&stock, 10_500_000, &DIVIDEND_CREDIT_IT), 75_000);
        // 全額が1,000万円超：100万×5% = 50,000
        assert_eq!(dividend_credit(&stock, 11_000_000, &DIVIDEND_CREDIT_IT), 50_000);
        assert_eq!(dividend_credit(&stock, 20_000_000, &DIVIDEND_CREDIT_IT), 50_000);

        // 剰余金100万＋投信40万、課税総所得金額等1,060万
        // 剰余金：1,060万 − 40万 − 1,000万 = 20万が超過 → 80万×10% + 20万×5% = 90,000
        // 投信：60万超過のうち40万全額 → 40万×2.5% = 10,000
        let both = DividendAmounts { stock: 1_000_000, fund: 400_000, ..Default::default() };
        assert_eq!(dividend_credit(&both, 10_600_000, &DIVIDEND_CREDIT_IT), 100_000);
    }

    #[test]
    fn undeclared_accounts_are_taxed_separately() {
        let st = StockInput {
            accounts: vec![
                account(StockAccountKind::Withholding, 1_000_000, false),
                account(StockAccountKind::Withholding, -300_000, false),
            ],
            ..Default::default()
        };
        let s = stock_income(&st);
        // 利益の口座だけ 100万×15.315% = 153,150、100万×5% = 50,000。損失の口座とは通算されない
        assert_eq!((s.wh_it, s.wh_rt, s.wh_base), (153_150, 50_000, 1_000_000));
        assert_eq!((s.gain, s.loss_next), (0, 0));
        assert!(!s.declares_gain);
        assert_eq!(s.net_gain, 700_000);
        assert!(s.warnings.iter().any(|w| w.contains("源泉徴収ありの特定口座の損失")));
    }

    #[test]
    fn declared_accounts_are_netted() {
        let st = StockInput {
            accounts: vec![
                account(StockAccountKind::Withholding, 1_000_000, true),
                account(StockAccountKind::Withholding, -300_000, true),
                // 一般口座・源泉徴収なしの特定口座は、チェックがなくても申告する
                account(StockAccountKind::General, 200_000, false),
                account(StockAccountKind::NoWithholding, -100_000, false),
            ],
            ..Default::default()
        };
        let s = stock_income(&st);
        assert_eq!(s.gain, 800_000);
        assert_eq!((s.wh_it, s.wh_rt), (0, 0));
        assert!(s.declares_gain);
        assert!(s.warnings.is_empty());
    }

    #[test]
    fn empty_general_account_does_not_require_filing() {
        let st = StockInput { accounts: vec![account(StockAccountKind::General, 0, false)], ..Default::default() };
        assert!(!stock_income(&st).declares_gain);
    }

    #[test]
    fn loss_offsets_separate_dividends_then_carries() {
        let st = StockInput {
            dividends: 300_000,
            dividend_mode: DividendMode::Separate,
            accounts: vec![account(StockAccountKind::Withholding, -500_000, true)],
            ..Default::default()
        };
        let s = stock_income(&st);
        assert_eq!((s.div_sep, s.gain, s.loss_next, s.separate_before_carry), (0, 0, 200_000, 0));
    }

    #[test]
    fn carryforward_order() {
        // 繰越損失80万：譲渡益50万 → 配当20万 の順に控除し、残り10万を翌年へ
        let st = StockInput {
            dividends_fund: 200_000,
            dividend_mode: DividendMode::Separate,
            accounts: vec![account(StockAccountKind::General, 500_000, false)],
            loss_carryforward: 800_000,
            ..Default::default()
        };
        let s = stock_income(&st);
        assert_eq!(s.separate_before_carry, 700_000);
        assert_eq!((s.gain, s.div_sep, s.loss_next), (0, 0, 100_000));
    }
}

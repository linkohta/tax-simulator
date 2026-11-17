//! 入力・出力のデータモデル（フロントエンドとは camelCase の JSON でやり取りする）

use serde::{Deserialize, Serialize};

use super::params::{
    CARE_RATE, CHILDCARE_SUPPORT_RATE, EMPLOYER_SPLIT, KYOKAI_KENPO_FALLBACK_PREFECTURE, KYOKAI_KENPO_RATES, TAX_YEAR,
};

// ───────────────────────────── 入力 ─────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TaxInput {
    /// 本人の年齢（2026年12月31日時点）
    pub age: u32,
    pub salary: SalaryInput,
    pub social_insurance: SocialInsuranceInput,
    pub family: FamilyInput,
    pub insurance: InsurancePremiumInput,
    pub pension_savings: PensionSavingsInput,
    pub medical: MedicalInput,
    pub donations: DonationInput,
    pub stocks: StockInput,
    pub housing_loan: HousingLoanInput,
    pub misc_income: MiscIncomeInput,
}

/// 雑所得（公的年金等、副業）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MiscIncomeInput {
    /// 公的年金等の収入金額（源泉徴収票の「支払金額」）
    pub pension_income: i64,
    /// 副業などの雑所得の収入金額
    pub side_revenue: i64,
    /// 同、必要経費
    pub side_expenses: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SalaryInput {
    /// 給与収入（年額・賞与込み、源泉徴収票の「支払金額」）
    pub annual_income: i64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SocialInsuranceMode {
    /// 給与から概算（協会けんぽ）
    #[default]
    Auto,
    /// 金額を直接入力（源泉徴収票の社会保険料等の金額）
    Manual,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HealthPlan {
    /// 協会けんぽ（都道府県別の料率を労使折半）
    #[default]
    Kyokai,
    /// 健康保険組合など（本人負担分の料率を入力）
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SocialInsuranceInput {
    pub mode: SocialInsuranceMode,
    /// Manual 時：給与天引きの社会保険料合計
    pub manual_amount: i64,
    /// Auto 時：月額給与（報酬月額：基本給＋諸手当＋通勤手当）
    pub monthly_salary: i64,
    /// Auto 時：賞与（1回ごとの支給額）
    pub bonuses: Vec<i64>,
    /// Auto 時：健康保険の種類
    pub health_plan: HealthPlan,
    /// Auto 時：協会けんぽの都道府県
    pub prefecture: String,
    /// Auto・健保組合など：健康保険料率の本人負担分（1/1000 %単位。4.5% → 4_500）
    pub custom_health_rate_x1000: i64,
    /// Auto・健保組合など：介護保険料率の本人負担分（1/1000 %単位）
    pub custom_care_rate_x1000: i64,
    /// Auto・健保組合など：子ども・子育て支援金率の本人負担分（1/1000 %単位）
    pub custom_childcare_support_rate_x1000: i64,
    /// Auto 時：雇用保険の対象か
    pub employment_insurance: bool,
    /// Auto 時：子ども・子育て支援金を負担する月数（2026年は4月分から。翌月徴収なら年内は8か月）
    pub childcare_support_months: u32,
    /// 給与天引き以外に支払った社会保険料（国民年金・国保・家族分など）
    pub other_amount: i64,
}

impl Default for SocialInsuranceInput {
    fn default() -> Self {
        Self {
            mode: SocialInsuranceMode::Auto,
            manual_amount: 0,
            monthly_salary: 0,
            bonuses: Vec::new(),
            health_plan: HealthPlan::Kyokai,
            prefecture: KYOKAI_KENPO_FALLBACK_PREFECTURE.into(),
            // 健保組合などの料率の初期値は、協会けんぽ（東京都）の本人負担分と同じにしておく
            custom_health_rate_x1000: KYOKAI_KENPO_RATES
                .iter()
                .find(|(p, _)| *p == KYOKAI_KENPO_FALLBACK_PREFECTURE)
                .map_or(0, |(_, r)| r / EMPLOYER_SPLIT),
            custom_care_rate_x1000: CARE_RATE / EMPLOYER_SPLIT,
            custom_childcare_support_rate_x1000: CHILDCARE_SUPPORT_RATE / EMPLOYER_SPLIT,
            employment_insurance: true,
            childcare_support_months: 8,
            other_amount: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Disability {
    #[default]
    None,
    /// 障害者
    General,
    /// 特別障害者
    Special,
    /// 同居特別障害者（扶養親族・配偶者のみ）
    SpecialCohabiting,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WidowStatus {
    #[default]
    None,
    /// 寡婦
    Widow,
    /// ひとり親（母）
    SingleMother,
    /// ひとり親（父）
    SingleFather,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct FamilyInput {
    pub has_spouse: bool,
    pub spouse: SpouseInput,
    pub dependents: Vec<DependentInput>,
    pub self_disability: Disability,
    pub widow_status: WidowStatus,
    pub working_student: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SpouseInput {
    /// 年齢（12/31時点）
    pub age: u32,
    /// 配偶者の給与収入
    pub salary_income: i64,
    /// 給与以外の所得（合計所得金額に算入されるもの）
    pub other_income: i64,
    pub disability: Disability,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DependentInput {
    pub name: String,
    /// 年齢（12/31時点）
    pub age: u32,
    pub salary_income: i64,
    pub other_income: i64,
    /// 直系尊属で同居（70歳以上の同居老親等）
    pub cohabiting_parent: bool,
    pub disability: Disability,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct InsurancePremiumInput {
    /// 一般生命保険料（新契約）
    pub life_new: i64,
    /// 一般生命保険料（旧契約）
    pub life_old: i64,
    /// 介護医療保険料
    pub medical_care: i64,
    /// 個人年金保険料（新契約）
    pub pension_new: i64,
    /// 個人年金保険料（旧契約）
    pub pension_old: i64,
    /// 地震保険料
    pub earthquake: i64,
    /// 旧長期損害保険料
    pub old_long_term: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PensionSavingsInput {
    /// iDeCo 掛金（年額）
    pub ideco: i64,
    /// 小規模企業共済掛金（年額）
    pub small_business_mutual_aid: i64,
    /// 企業型DCのマッチング拠出（年額）
    pub corporate_dc_matching: i64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MedicalMode {
    #[default]
    Normal,
    /// セルフメディケーション税制
    SelfMedication,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MedicalInput {
    pub mode: MedicalMode,
    /// 支払った医療費
    pub expenses: i64,
    /// 保険金などで補てんされる金額
    pub reimbursed: i64,
    /// 特定一般用医薬品等購入費（セルフメディケーション）
    pub otc_purchases: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DonationInput {
    /// ふるさと納税（都道府県・市区町村への寄附）の一覧
    pub furusato: Vec<FurusatoEntry>,
    /// ワンストップ特例を利用する（確定申告をしない）
    pub one_stop: bool,
    /// その他の寄附（国・認定NPO等）で所得税の寄附金控除対象のもの
    pub other_income_tax_only: i64,
    /// その他の寄附のうち、住民税の条例指定寄附にも該当するもの（都道府県・市区町村の両方指定を想定）
    pub other_resident_designated: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct FurusatoEntry {
    pub name: String,
    pub amount: i64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DividendMode {
    /// 申告不要（源泉徴収で完結）
    #[default]
    NoDeclare,
    /// 総合課税（配当控除あり）
    Comprehensive,
    /// 申告分離課税（譲渡損失と損益通算可）
    Separate,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StockAccountKind {
    /// 特定口座（源泉徴収あり）：申告するかどうかを選べる
    #[default]
    Withholding,
    /// 特定口座（源泉徴収なし）：申告が必要
    NoWithholding,
    /// 一般口座：申告が必要
    General,
}

/// 上場株式等の口座ごとの譲渡損益
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct StockAccount {
    pub name: String,
    pub kind: StockAccountKind,
    /// 年間の譲渡損益（マイナスは損失）
    pub capital_gain: i64,
    /// 源泉徴収ありの特定口座の損益を申告する（ほかの種類の口座では使わない）
    pub declare: bool,
    /// 源泉徴収ありの特定口座に受け入れた配当等（`StockInput` の配当等の内数）。
    /// 申告しなければ、口座内で譲渡損失と通算される
    pub dividends_received: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct StockInput {
    /// 上場株式等の配当等（税引前）のうち剰余金の配当等（国内株式・国内株式ETFなど。配当控除率10%）
    pub dividends: i64,
    /// 同、証券投資信託の収益の分配（外貨建資産・株式以外の割合が50%以下。配当控除率5%）
    pub dividends_fund: i64,
    /// 同、外貨建等証券投資信託の収益の分配（同割合が50%超75%以下。配当控除率2.5%）
    pub dividends_foreign_fund: i64,
    /// 同、配当控除の対象外（外国株式、J-REIT、同割合75%超の投資信託など）
    pub dividends_no_credit: i64,
    pub dividend_mode: DividendMode,
    /// 口座ごとの譲渡損益
    pub accounts: Vec<StockAccount>,
    /// 前年以前から繰り越された上場株式等の譲渡損失
    pub loss_carryforward: i64,
    /// 国外所得金額（外国株式の配当など。外国所得税を引く前）
    pub foreign_income: i64,
    /// その年に納付した外国所得税額
    pub foreign_tax_paid: i64,
    /// 旧形式：全口座を合算した譲渡損益。読込時に `accounts` へ移す（[`TaxInput::normalized`]）
    #[serde(skip_serializing)]
    pub capital_gain: i64,
    /// 旧形式：譲渡損益を申告する
    #[serde(skip_serializing)]
    pub declare_capital_gain: bool,
}

/// 旧形式の譲渡損益を移した口座の名前
const LEGACY_STOCK_ACCOUNT_NAME: &str = "特定口座";

impl TaxInput {
    /// 旧形式の保存ファイルの項目を現在の形式に直したものを返す
    pub fn normalized(&self) -> TaxInput {
        let mut out = self.clone();
        let st = &mut out.stocks;
        if st.capital_gain != 0 || st.declare_capital_gain {
            st.accounts.push(StockAccount {
                name: LEGACY_STOCK_ACCOUNT_NAME.into(),
                kind: StockAccountKind::Withholding,
                capital_gain: st.capital_gain,
                declare: st.declare_capital_gain,
                dividends_received: 0,
            });
            st.capital_gain = 0;
            st.declare_capital_gain = false;
        }
        out
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HousingLoanMode {
    /// 控除可能額を入力する
    #[default]
    Manual,
    /// 年末残高・居住開始年・住宅の区分から計算する
    Auto,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HousingType {
    /// 認定長期優良住宅・認定低炭素住宅
    Certified,
    /// ZEH水準省エネ住宅
    Zeh,
    /// 省エネ基準適合住宅
    #[default]
    EnergySaving,
    /// その他の住宅
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct HousingLoanInput {
    pub mode: HousingLoanMode,
    /// Auto 時：住宅借入金等の年末残高（自分の持分・居住用部分に対応する額）
    pub year_end_balance: i64,
    /// Auto 時：居住を開始した年（西暦）
    pub move_in_year: u32,
    /// Auto 時：住宅の区分
    pub housing_type: HousingType,
    /// Auto 時：中古（既存住宅）か。false は新築・買取再販
    pub existing_house: bool,
    /// Auto 時：特例対象個人（居住年の年末に、19歳未満の扶養親族がいる、または夫婦のどちらかが40歳未満）
    pub child_rearing: bool,
    /// Manual 時：住宅借入金等特別控除の控除可能額（年末残高×0.7% などで計算済みの額）
    pub credit_amount: i64,
    /// 令和8年以後に居住を開始した（住民税の控除限度額の計算が異なる）
    pub moved_in_2026_or_later: bool,
    /// 住民税の控除限度額が7%・136,500円になる入居
    /// （平成26年4月〜令和3年12月入居で消費税率8%・10%の特定取得など）
    pub specific_acquisition: bool,
}

impl Default for HousingLoanInput {
    fn default() -> Self {
        Self {
            mode: HousingLoanMode::Manual,
            year_end_balance: 0,
            move_in_year: TAX_YEAR,
            housing_type: HousingType::default(),
            existing_house: false,
            child_rearing: false,
            credit_amount: 0,
            moved_in_2026_or_later: false,
            specific_acquisition: false,
        }
    }
}

// ───────────────────────────── 出力 ─────────────────────────────

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxResult {
    pub income: IncomeBreakdown,
    pub social_insurance: SocialInsuranceBreakdown,
    pub income_tax: TaxDetail,
    pub resident_tax: ResidentTaxDetail,
    pub stock_withholding: StockWithholding,
    pub furusato: FurusatoResult,
    /// 株式の課税方式ごとの税額（配当も源泉徴収ありの口座の損益もなければ空）
    pub stock_options: Vec<StockOption>,
    /// 控除ごとの節税額（効果のあるものだけ）
    pub savings: Vec<SavingItem>,
    pub summary: Summary,
    pub warnings: Vec<String>,
}

/// 給与収入を変えたときの内訳（グラフの1点）。4つの合計は収入の合計に一致する
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurvePoint {
    /// 給与収入
    pub salary: i64,
    pub take_home: i64,
    pub social_insurance: i64,
    /// 所得税及び復興特別所得税（申告不要とした株式の源泉徴収税の所得税分を含む）
    pub income_tax: i64,
    /// 住民税（同、住民税分を含む）
    pub resident_tax: i64,
}

/// ある控除がなかった場合と比べて減っている税額
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavingItem {
    pub label: String,
    /// 所得税及び復興特別所得税の減少額
    pub income_tax: i64,
    /// 住民税の減少額
    pub resident_tax: i64,
    pub total: i64,
}

/// 株式の課税方式を1つ選んだときの税額
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StockOption {
    pub dividend_mode: DividendMode,
    /// 源泉徴収ありの特定口座の譲渡損益を申告するか（損益のある該当口座がなければ `None`）
    pub declare_withholding_accounts: Option<bool>,
    pub income_tax: i64,
    pub resident_tax: i64,
    pub stock_withholding: i64,
    /// 所得税＋住民税＋申告不要とした分の源泉徴収税
    pub total_tax: i64,
    pub loss_carried_to_next_year: i64,
    /// 現在の入力での税額合計との差（マイナスなら有利）
    pub diff_from_current: i64,
    /// 現在の入力と同じ方式か
    pub is_current: bool,
    /// 税額合計が最も少ない方式か
    pub is_best: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncomeBreakdown {
    pub salary_income: i64,
    pub salary_deduction: i64,
    pub income_adjustment_deduction: i64,
    /// 給与所得
    pub salary_net: i64,
    /// 公的年金等に係る雑所得
    pub pension_net: i64,
    /// 副業の雑所得（収入 − 必要経費。赤字ならマイナス）
    pub side_net: i64,
    /// 雑所得の金額（公的年金等と副業を通算。赤字は0）
    pub misc_income: i64,
    /// 総合課税の配当所得
    pub dividend_comprehensive: i64,
    /// 申告分離の上場株式等の配当所得等（損益通算後）
    pub dividend_separate: i64,
    /// 申告分離の上場株式等の譲渡所得等（損益通算・繰越控除後）
    pub capital_gain_separate: i64,
    /// 総所得金額等（繰越控除後）
    pub total_income: i64,
    /// 合計所得金額（扶養判定・基礎控除判定用）
    pub total_income_before_carryover: i64,
    /// 翌年へ繰り越す譲渡損失
    pub loss_carried_to_next_year: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SocialInsuranceBreakdown {
    pub health: i64,
    pub care: i64,
    pub childcare_support: i64,
    pub pension: i64,
    pub employment: i64,
    pub other: i64,
    pub total: i64,
    pub standard_monthly_health: i64,
    pub standard_monthly_pension: i64,
    pub estimated: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeductionItem {
    pub label: String,
    pub amount: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxDetail {
    pub deductions: Vec<DeductionItem>,
    pub total_deductions: i64,
    /// 課税総所得金額（1,000円未満切捨て）
    pub taxable_ordinary: i64,
    /// 課税分離所得（配当・譲渡）
    pub taxable_separate: i64,
    pub tax_on_ordinary: i64,
    pub tax_on_separate: i64,
    pub credits: Vec<DeductionItem>,
    /// 基準所得税額
    pub base_tax: i64,
    /// 復興特別所得税
    pub reconstruction_tax: i64,
    /// 外国税額控除（所得税分＋復興特別所得税分）
    pub foreign_tax_credit: i64,
    /// 所得税及び復興特別所得税（100円未満切捨て）
    pub total: i64,
    /// 限界税率（%）
    pub marginal_rate: f64,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResidentTaxDetail {
    pub deductions: Vec<DeductionItem>,
    pub total_deductions: i64,
    pub taxable_ordinary: i64,
    pub taxable_separate: i64,
    pub income_levy_before_credits: i64,
    pub credits: Vec<DeductionItem>,
    /// 所得割（市町村＋道府県）
    pub income_levy: i64,
    /// 均等割（森林環境税を含む）
    pub per_capita: i64,
    pub total: i64,
    pub income_levy_exempt: bool,
    pub per_capita_exempt: bool,
    /// 給与天引き（特別徴収）の場合の6月分の額
    pub monthly_first: i64,
    /// 同、7月〜翌年5月分の各月の額
    pub monthly_rest: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StockWithholding {
    /// 申告不要とした配当・譲渡益に係る源泉徴収税（所得税・復興税分）
    pub income_tax: i64,
    /// 同（住民税分）
    pub resident_tax: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FurusatoResult {
    /// 現在入力されたふるさと納税合計
    pub current_total: i64,
    /// 自己負担2,000円で収まる寄附上限額の目安
    pub limit: i64,
    /// 現在の寄附での実質自己負担額
    pub current_burden: i64,
    /// 所得税からの軽減額
    pub income_tax_reduction: i64,
    /// 住民税からの軽減額
    pub resident_tax_reduction: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub gross_income: i64,
    pub social_insurance: i64,
    pub income_tax: i64,
    pub resident_tax: i64,
    pub stock_withholding: i64,
    pub total_tax: i64,
    /// 手取り（給与＋株式の税引後）−iDeCo等の拠出は含めない
    pub take_home: i64,
    /// 手取りの月平均（賞与・株式を含む年額 ÷ 12）
    pub take_home_monthly: i64,
    pub effective_tax_rate: f64,
    pub filing_required: bool,
}

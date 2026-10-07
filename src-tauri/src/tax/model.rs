//! 入力・出力のデータモデル（フロントエンドとは camelCase の JSON でやり取りする）

use serde::{Deserialize, Serialize};

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
    /// Auto 時：協会けんぽの都道府県
    pub prefecture: String,
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
            prefecture: "東京都".into(),
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct StockInput {
    /// 上場株式等の配当等（税引前）
    pub dividends: i64,
    pub dividend_mode: DividendMode,
    /// 上場株式等の譲渡損益（年間通算後、マイナスは損失）
    pub capital_gain: i64,
    /// 譲渡所得を申告する（源泉徴収なし口座・一般口座の場合や損益通算時は申告）
    pub declare_capital_gain: bool,
    /// 前年以前から繰り越された上場株式等の譲渡損失
    pub loss_carryforward: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct HousingLoanInput {
    /// 住宅借入金等特別控除の控除可能額（年末残高×0.7% などで計算済みの額）
    pub credit_amount: i64,
    /// 令和8年以後に居住を開始した（住民税の控除限度額の計算が異なる）
    pub moved_in_2026_or_later: bool,
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
    pub summary: Summary,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncomeBreakdown {
    pub salary_income: i64,
    pub salary_deduction: i64,
    pub income_adjustment_deduction: i64,
    /// 給与所得
    pub salary_net: i64,
    /// 総合課税の配当所得
    pub dividend_comprehensive: i64,
    /// 申告分離の上場株式等の配当所得等（損益通算後）
    pub dividend_separate: i64,
    /// 申告分離の上場株式等の譲渡所得等（損益通算・繰越控除後）
    pub capital_gain_separate: i64,
    /// 総所得金額等（繰越控除後）
    pub total_income: i64,
    /// 合計所得金額（扶養判定・基礎控除判定用）
    pub total_income_for_tests: i64,
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
    pub effective_tax_rate: f64,
    pub filing_required: bool,
}

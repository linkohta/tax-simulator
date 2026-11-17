//! 税制パラメータ（令和8年分所得税・令和9年度住民税／2026年度社会保険料）
//!
//! 金額は円。率は注記した単位の整数で持つ（浮動小数点の誤差を避けるため）。

// ───────────── 給与所得控除 ─────────────

/// 給与所得控除の最低保障額（所得税：令和8・9年分は措法29条の4により74万円）
pub const SALARY_DEDUCTION_MIN_IT: i64 = 740_000;
/// 給与所得控除の最低保障額（住民税：令和9・10年度分は74万円）
pub const SALARY_DEDUCTION_MIN_RT: i64 = 740_000;
/// 給与所得の金額の特例（措法29条の4第2項の端数表）：(収入下限, 収入上限未満, 給与所得)
pub static SALARY_NET_SPECIAL: &[(i64, i64, i64)] = &[
    (2_191_000, 2_193_000, 1_451_000),
    (2_193_000, 2_196_000, 1_453_000),
    (2_196_000, 2_200_000, 1_456_000),
];

/// 所得金額調整控除（子ども・特別障害者等を有する者等）：給与収入がこの額を超えると対象
pub const INCOME_ADJUSTMENT_FROM: i64 = 8_500_000;
/// 同：計算に使う給与収入の上限
pub const INCOME_ADJUSTMENT_SALARY_CAP: i64 = 10_000_000;
/// 同：控除率（%）
pub const INCOME_ADJUSTMENT_RATE_PCT: i64 = 10;

// ───────────── 基礎控除 ─────────────
// (合計所得金額の上限, 控除額)。表にない（上限超）場合は0。

/// 所得税：本則62万円＋措法41条の16の2の特例加算（令和8・9年分：489万以下+42万、655万以下+5万）
pub static BASIC_DEDUCTION_IT: &[(i64, i64)] = &[
    (4_890_000, 1_040_000),
    (6_550_000, 670_000),
    (23_500_000, 620_000),
    (24_000_000, 480_000),
    (24_500_000, 320_000),
    (25_000_000, 160_000),
];

pub static BASIC_DEDUCTION_RT: &[(i64, i64)] = &[
    (24_000_000, 430_000),
    (24_500_000, 290_000),
    (25_000_000, 150_000),
];

// ───────────── 配偶者・扶養 ─────────────

/// 同一生計配偶者・扶養親族の合計所得金額の上限
pub const DEPENDENT_INCOME_LIMIT: i64 = 620_000;

/// 配偶者控除 [納税者900万以下, 950万以下, 1000万以下]
pub const SPOUSE_IT: [i64; 3] = [380_000, 260_000, 130_000];
pub const SPOUSE_ELDERLY_IT: [i64; 3] = [480_000, 320_000, 160_000];
pub const SPOUSE_RT: [i64; 3] = [330_000, 220_000, 110_000];
pub const SPOUSE_ELDERLY_RT: [i64; 3] = [380_000, 260_000, 130_000];
/// 調整控除用の人的控除差（配偶者控除）
pub const SPOUSE_DIFF: [i64; 3] = [50_000, 40_000, 20_000];
pub const SPOUSE_DIFF_ELDERLY: [i64; 3] = [100_000, 60_000, 30_000];

pub struct SpouseSpecialRow {
    /// 配偶者の合計所得金額の上限
    pub upper: i64,
    pub it: [i64; 3],
    pub rt: [i64; 3],
    /// 調整控除用の人的控除差
    pub diff: [i64; 3],
}

macro_rules! ss {
    ($u:expr, [$a:expr, $b:expr, $c:expr], [$d:expr, $e:expr, $f:expr], [$g:expr, $h:expr, $i:expr]) => {
        SpouseSpecialRow { upper: $u, it: [$a, $b, $c], rt: [$d, $e, $f], diff: [$g, $h, $i] }
    };
}

/// 配偶者特別控除（配偶者の合計所得62万円超133万円以下）
/// 人的控除差は配偶者所得55万円未満の場合のみのため、令和9年度は全て0
pub static SPOUSE_SPECIAL: &[SpouseSpecialRow] = &[
    ss!(950_000, [380_000, 260_000, 130_000], [330_000, 220_000, 110_000], [0, 0, 0]),
    ss!(1_000_000, [360_000, 240_000, 120_000], [330_000, 220_000, 110_000], [0, 0, 0]),
    ss!(1_050_000, [310_000, 210_000, 110_000], [310_000, 210_000, 110_000], [0, 0, 0]),
    ss!(1_100_000, [260_000, 180_000, 90_000], [260_000, 180_000, 90_000], [0, 0, 0]),
    ss!(1_150_000, [210_000, 140_000, 70_000], [210_000, 140_000, 70_000], [0, 0, 0]),
    ss!(1_200_000, [160_000, 110_000, 60_000], [160_000, 110_000, 60_000], [0, 0, 0]),
    ss!(1_250_000, [110_000, 80_000, 40_000], [110_000, 80_000, 40_000], [0, 0, 0]),
    ss!(1_300_000, [60_000, 40_000, 20_000], [60_000, 40_000, 20_000], [0, 0, 0]),
    ss!(1_330_000, [30_000, 20_000, 10_000], [30_000, 20_000, 10_000], [0, 0, 0]),
];

pub const DEP_GENERAL_IT: i64 = 380_000;
pub const DEP_GENERAL_RT: i64 = 330_000;
pub const DEP_SPECIFIC_IT: i64 = 630_000;
pub const DEP_SPECIFIC_RT: i64 = 450_000;
pub const DEP_ELDERLY_IT: i64 = 480_000;
pub const DEP_ELDERLY_RT: i64 = 380_000;
pub const DEP_ELDERLY_COHAB_IT: i64 = 580_000;
pub const DEP_ELDERLY_COHAB_RT: i64 = 450_000;
pub const DIFF_DEP_GENERAL: i64 = 50_000;
pub const DIFF_DEP_SPECIFIC: i64 = 180_000;
pub const DIFF_DEP_ELDERLY: i64 = 100_000;
pub const DIFF_DEP_ELDERLY_COHAB: i64 = 130_000;

pub struct SpecificRelativeRow {
    pub upper: i64,
    pub it: i64,
    pub rt: i64,
    pub diff: i64,
}

/// 特定親族特別控除（19〜22歳、合計所得62万円超123万円以下）。調整控除の人的控除差の対象外
pub static SPECIFIC_RELATIVE: &[SpecificRelativeRow] = &[
    SpecificRelativeRow { upper: 850_000, it: 630_000, rt: 450_000, diff: 0 },
    SpecificRelativeRow { upper: 900_000, it: 610_000, rt: 450_000, diff: 0 },
    SpecificRelativeRow { upper: 950_000, it: 510_000, rt: 450_000, diff: 0 },
    SpecificRelativeRow { upper: 1_000_000, it: 410_000, rt: 410_000, diff: 0 },
    SpecificRelativeRow { upper: 1_050_000, it: 310_000, rt: 310_000, diff: 0 },
    SpecificRelativeRow { upper: 1_100_000, it: 210_000, rt: 210_000, diff: 0 },
    SpecificRelativeRow { upper: 1_150_000, it: 110_000, rt: 110_000, diff: 0 },
    SpecificRelativeRow { upper: 1_200_000, it: 60_000, rt: 60_000, diff: 0 },
    SpecificRelativeRow { upper: 1_230_000, it: 30_000, rt: 30_000, diff: 0 },
];

pub const DISABLED_IT: i64 = 270_000;
pub const DISABLED_RT: i64 = 260_000;
pub const SPECIAL_DISABLED_IT: i64 = 400_000;
pub const SPECIAL_DISABLED_RT: i64 = 300_000;
pub const COHAB_SPECIAL_DISABLED_IT: i64 = 750_000;
pub const COHAB_SPECIAL_DISABLED_RT: i64 = 530_000;

pub const WIDOW_INCOME_LIMIT: i64 = 5_000_000;
pub const WIDOW_IT: i64 = 270_000;
pub const WIDOW_RT: i64 = 260_000;
pub const DIFF_WIDOW: i64 = 10_000;
pub const SINGLE_PARENT_IT: i64 = 350_000;
pub const SINGLE_PARENT_RT: i64 = 300_000;
/// ひとり親の人的控除差（母5万円・父1万円）
pub const DIFF_SINGLE_PARENT_MOTHER: i64 = 50_000;
pub const DIFF_SINGLE_PARENT_FATHER: i64 = 10_000;

pub const WORKING_STUDENT_INCOME_LIMIT: i64 = 890_000;
pub const WORKING_STUDENT_IT: i64 = 270_000;
pub const WORKING_STUDENT_RT: i64 = 260_000;
pub const DIFF_WORKING_STUDENT: i64 = 10_000;

/// 基礎控除の人的控除差（調整控除・特例控除割合の判定用）
pub const PERSONAL_DIFF_BASIC: i64 = 50_000;
/// ふるさと納税の人的控除差調整額・住宅ローン控除の住民税限度額で、
/// 所得税の基礎控除額のうちこの額を超える部分を加算する（地方税法37条の2第11項等）
pub const BASIC_DEDUCTION_EXCESS_BASE: i64 = 480_000;

// ───────────── 保険料控除 ─────────────

/// 令和8年分：23歳未満の扶養親族がいる場合の一般生命保険料（新契約）控除上限
pub const LIFE_GENERAL_CAP_SPECIAL_IT: i64 = 60_000;

// ───────────── 所得税 ─────────────

pub struct Bracket {
    pub upper: i64,
    pub rate_pct: i64,
    pub deduction: i64,
}

pub static INCOME_TAX_BRACKETS: &[Bracket] = &[
    Bracket { upper: 1_949_000, rate_pct: 5, deduction: 0 },
    Bracket { upper: 3_299_000, rate_pct: 10, deduction: 97_500 },
    Bracket { upper: 6_949_000, rate_pct: 20, deduction: 427_500 },
    Bracket { upper: 8_999_000, rate_pct: 23, deduction: 636_000 },
    Bracket { upper: 17_999_000, rate_pct: 33, deduction: 1_536_000 },
    Bracket { upper: 39_999_000, rate_pct: 40, deduction: 2_796_000 },
    Bracket { upper: i64::MAX, rate_pct: 45, deduction: 4_796_000 },
];

/// 復興特別所得税 2.1%（1/1000 %単位）
pub const RECONSTRUCTION_RATE_X1000: i64 = 2_100;

/// 上場株式等の源泉徴収税率：所得税・復興税 15.315%（1/1000 %単位）、住民税 5%
pub const STOCK_IT_RATE_X1000: i64 = 15_315;
pub const STOCK_RT_RATE_PCT: i64 = 5;

/// 配当控除で率が変わる課税総所得金額等の区切り（所法92条）
pub const DIVIDEND_CREDIT_THRESHOLD: i64 = 10_000_000;
/// 配当控除率（1/1000 %単位）。
/// 行は [剰余金の配当等, 証券投資信託, 外貨建等証券投資信託]、
/// 列は [課税総所得金額等1,000万円以下の部分, 超える部分]。
/// 所得税：所法92条・措法9条。住民税：地方税法附則5条
pub const DIVIDEND_CREDIT_IT: [[i64; 2]; 3] = [[10_000, 5_000], [5_000, 2_500], [2_500, 1_250]];
pub const DIVIDEND_CREDIT_RT_MUNI: [[i64; 2]; 3] = [[1_600, 800], [800, 400], [400, 200]];
pub const DIVIDEND_CREDIT_RT_PREF: [[i64; 2]; 3] = [[1_200, 600], [600, 300], [300, 150]];

/// 外国税額控除の住民税の控除限度額：所得税の控除限度額に対する率（%）。
/// 道府県民税12%（地方税法施行令7条の19）、市町村民税18%（同48条の9の2）。指定都市の6%・24%は扱わない
pub const FOREIGN_TAX_CREDIT_RT_PREF_PCT: i64 = 12;
pub const FOREIGN_TAX_CREDIT_RT_MUNI_PCT: i64 = 18;

// ───────────── 住民税 ─────────────

/// 均等割（市町村3,000＋道府県1,000＋森林環境税1,000）
pub const PER_CAPITA_TOTAL: i64 = 5_000;
/// 住宅ローン控除の住民税からの控除限度額（所得税の課税総所得金額等×率、上限あり）
pub struct HousingLoanResidentLimit {
    /// 所得税の課税総所得金額等に掛ける率（%）
    pub rate_pct: i64,
    /// 控除限度額の上限
    pub cap: i64,
}
/// 原則：5%・97,500円（地方税法附則5条の4の2）
pub const HOUSING_LOAN_RT_STANDARD: HousingLoanResidentLimit = HousingLoanResidentLimit { rate_pct: 5, cap: 97_500 };
/// 平成26年4月〜令和3年12月入居で消費税率8%・10%の特定取得など：7%・136,500円（同条）
pub const HOUSING_LOAN_RT_SPECIFIC: HousingLoanResidentLimit = HousingLoanResidentLimit { rate_pct: 7, cap: 136_500 };

/// 「極めて高い水準の所得に対する負担の適正化措置」の対象となる基準所得金額（措法41条の19）。
/// 令和7・8年分は3億3,000万円（令和8年度改正により、令和9年分以後は1億6,500万円）
pub const MINIMUM_TAX_THRESHOLD: i64 = 330_000_000;
/// 障害者・未成年・寡婦・ひとり親の非課税限度（合計所得）
pub const RT_PROTECTED_INCOME_LIMIT: i64 = 1_350_000;

/// 均等割の非課税限度額（1級地の標準：合計所得金額）
pub fn per_capita_exempt_limit(count: i64, has_deps: bool) -> i64 {
    if has_deps {
        350_000 * count + 100_000 + 210_000
    } else {
        450_000
    }
}

/// 所得割の非課税限度額（1級地の標準：総所得金額等）
pub fn income_levy_exempt_limit(count: i64, has_deps: bool) -> i64 {
    if has_deps {
        350_000 * count + 100_000 + 320_000
    } else {
        450_000
    }
}

/// ふるさと納税の特例控除割合の区分（課税総所得金額−人的控除差調整額）
pub struct FurusatoBand {
    pub upper: i64,
    /// 特例控除割合（1/1000 %単位）
    pub special_x1000: i64,
    /// 申告特例控除の割合（分子, 分母）。法令上の表は5区分で900万円超は同率
    pub filing_ratio: (i64, i64),
}

pub static FURUSATO_BANDS: &[FurusatoBand] = &[
    FurusatoBand { upper: 1_950_000, special_x1000: 84_895, filing_ratio: (5_105, 84_895) },
    FurusatoBand { upper: 3_300_000, special_x1000: 79_790, filing_ratio: (10_210, 79_790) },
    FurusatoBand { upper: 6_950_000, special_x1000: 69_580, filing_ratio: (20_420, 69_580) },
    FurusatoBand { upper: 9_000_000, special_x1000: 66_517, filing_ratio: (23_483, 66_517) },
    FurusatoBand { upper: 18_000_000, special_x1000: 56_307, filing_ratio: (33_693, 56_307) },
    FurusatoBand { upper: 40_000_000, special_x1000: 49_160, filing_ratio: (33_693, 56_307) },
    FurusatoBand { upper: i64::MAX, special_x1000: 44_055, filing_ratio: (33_693, 56_307) },
];
/// 課税総所得金額−人的控除差調整額がマイナスの場合
pub static FURUSATO_BAND_NEGATIVE: FurusatoBand = FurusatoBand { upper: 0, special_x1000: 90_000, filing_ratio: (0, 1) };
/// 課税総所得がなく分離課税所得のみの場合（90% − 15.315%）
pub static FURUSATO_BAND_SEPARATE: FurusatoBand = FurusatoBand { upper: 0, special_x1000: 74_685, filing_ratio: (15_315, 74_685) };

// ───────────── 社会保険料（2026年度） ─────────────

/// 協会けんぽ 令和8年度 都道府県単位保険料率（令和8年3月分〜、1/1000 %単位、労使合計）
pub static KYOKAI_KENPO_RATES: &[(&str, i64)] = &[
    ("北海道", 10_280),
    ("青森県", 9_850),
    ("岩手県", 9_510),
    ("宮城県", 10_100),
    ("秋田県", 10_010),
    ("山形県", 9_750),
    ("福島県", 9_500),
    ("茨城県", 9_520),
    ("栃木県", 9_820),
    ("群馬県", 9_680),
    ("埼玉県", 9_670),
    ("千葉県", 9_730),
    ("東京都", 9_850),
    ("神奈川県", 9_920),
    ("新潟県", 9_210),
    ("富山県", 9_590),
    ("石川県", 9_700),
    ("福井県", 9_710),
    ("山梨県", 9_550),
    ("長野県", 9_630),
    ("岐阜県", 9_800),
    ("静岡県", 9_610),
    ("愛知県", 9_930),
    ("三重県", 9_770),
    ("滋賀県", 9_880),
    ("京都府", 9_890),
    ("大阪府", 10_130),
    ("兵庫県", 10_120),
    ("奈良県", 9_910),
    ("和歌山県", 10_060),
    ("鳥取県", 9_860),
    ("島根県", 9_940),
    ("岡山県", 10_050),
    ("広島県", 9_780),
    ("山口県", 10_150),
    ("徳島県", 10_240),
    ("香川県", 10_020),
    ("愛媛県", 9_980),
    ("高知県", 10_050),
    ("福岡県", 10_110),
    ("佐賀県", 10_550),
    ("長崎県", 10_060),
    ("熊本県", 10_080),
    ("大分県", 10_080),
    ("宮崎県", 9_770),
    ("鹿児島県", 10_130),
    ("沖縄県", 9_440),
];

/// 労使折半：労使合計の料率を本人負担分にするための除数
pub const EMPLOYER_SPLIT: i64 = 2;
/// 介護保険料率（40〜64歳、労使合計）
pub const CARE_RATE: i64 = 1_620;
/// 介護保険第2号被保険者の年齢（40歳以上65歳未満。介護保険法9条2号）
pub const CARE_AGE_FROM: u32 = 40;
pub const CARE_AGE_UNTIL: u32 = 65;
/// 健康保険の被保険者はこの年齢未満（75歳から後期高齢者医療制度。健康保険法3条1項7号）
pub const HEALTH_AGE_UNTIL: u32 = 75;
/// 厚生年金保険の被保険者はこの年齢未満（厚生年金保険法9条）
pub const PENSION_AGE_UNTIL: u32 = 70;
/// 子ども・子育て支援金率（労使合計）
pub const CHILDCARE_SUPPORT_RATE: i64 = 230;
/// 厚生年金保険料率 18.3%
pub const PENSION_RATE: i64 = 18_300;
pub const PENSION_STANDARD_MIN: i64 = 88_000;
pub const PENSION_STANDARD_MAX: i64 = 650_000;
/// 健康保険の標準賞与額の年度累計上限
pub const HEALTH_BONUS_ANNUAL_CAP: i64 = 5_730_000;
/// 厚生年金の標準賞与額の1回あたり上限
pub const PENSION_BONUS_PER_PAYMENT_CAP: i64 = 1_500_000;
/// 雇用保険料率（労働者負担、1/10000単位：5/1000 → 50）
pub const EMPLOYMENT_RATE: i64 = 50;

/// 住民税の特別徴収（給与天引き）の回数：6月から翌年5月までの12回（地方税法321条の5）
pub const RESIDENT_INSTALLMENTS: i64 = 12;
/// 1年の月数
pub const MONTHS_PER_YEAR: i64 = 12;

// ───────────── 率の単位・端数 ─────────────

/// 「1/1000 %単位」の率を金額に掛けるときの分母（9.85% = 9_850 / 100_000）
pub const RATE_X1000_DENOM: i64 = 100_000;
/// 課税所得金額の端数：1,000円未満切捨て（国税通則法118条、地方税法20条の4の2）
pub const TAXABLE_INCOME_UNIT: i64 = 1_000;
/// 税額の端数：100円未満切捨て（国税通則法119条、地方税法20条の4の2）
pub const TAX_UNIT: i64 = 100;

// ───────────── 給与所得控除の算式 ─────────────

/// 給与所得控除額 = 収入 × 率 + 加算額（所法28条3項）
pub struct SalaryDeductionRow {
    /// この収入以下に適用
    pub upper: i64,
    pub rate_pct: i64,
    pub add: i64,
}
pub static SALARY_DEDUCTION_TABLE: &[SalaryDeductionRow] = &[
    SalaryDeductionRow { upper: 1_800_000, rate_pct: 40, add: -100_000 },
    SalaryDeductionRow { upper: 3_600_000, rate_pct: 30, add: 80_000 },
    SalaryDeductionRow { upper: 6_600_000, rate_pct: 20, add: 440_000 },
    SalaryDeductionRow { upper: 8_500_000, rate_pct: 10, add: 1_100_000 },
];
/// 給与所得控除額の上限（収入850万円超）
pub const SALARY_DEDUCTION_MAX: i64 = 1_950_000;
/// 収入がこの額未満なら所得税法別表第五による（収入を刻みの下端に切り下げて算式に当てはめる）
pub const SALARY_TABLE_UNTIL: i64 = 6_600_000;
/// 別表第五の刻み
pub const SALARY_TABLE_STEP: i64 = 4_000;

// ───────────── 年齢・所得の区分 ─────────────

/// 配偶者控除・配偶者特別控除の、納税者の合計所得金額の区分の上限（これを超えると控除なし）
pub const SPOUSE_TAXPAYER_TIERS: [i64; 3] = [9_000_000, 9_500_000, 10_000_000];
/// 控除対象扶養親族は16歳以上
pub const DEP_AGE_FROM: u32 = 16;
/// 特定扶養親族・特定親族特別控除は19歳以上23歳未満
pub const DEP_SPECIFIC_AGE_FROM: u32 = 19;
/// 23歳未満（特定扶養親族の上限、所得金額調整控除・生命保険料控除の特例の判定）
pub const YOUNG_DEPENDENT_AGE_UNTIL: u32 = 23;
/// 老人扶養親族・老人控除対象配偶者は70歳以上
pub const ELDERLY_AGE_FROM: u32 = 70;
/// 住民税の非課税判定での未成年者（18歳未満）
pub const RT_MINOR_AGE_UNTIL: u32 = 18;

// ───────────── 生命保険料控除・地震保険料控除・医療費控除 ─────────────

/// 生命保険料控除の算式の区切り（所法76条、地方税法34条1項5号）。
/// `full_until` 以下は全額、`half_until` 以下は 1/2 + 定額、`quarter_until` 以下は 1/4 + 定額、超えると上限。
/// 定額と上限は、区切りで金額が連続することから決まる（deductions::life_amount）。
pub struct LifeInsuranceTier {
    pub full_until: i64,
    pub half_until: i64,
    pub quarter_until: i64,
}
/// 所得税・新契約（上限4万円）
pub const LIFE_NEW_IT: LifeInsuranceTier = LifeInsuranceTier { full_until: 20_000, half_until: 40_000, quarter_until: 80_000 };
/// 所得税・一般生命保険料の新契約で23歳未満の扶養親族がいる場合（上限6万円。令和8年分の特例）
pub const LIFE_NEW_SPECIAL_IT: LifeInsuranceTier = LifeInsuranceTier { full_until: 30_000, half_until: 60_000, quarter_until: 120_000 };
/// 所得税・旧契約（上限5万円）
pub const LIFE_OLD_IT: LifeInsuranceTier = LifeInsuranceTier { full_until: 25_000, half_until: 50_000, quarter_until: 100_000 };
/// 住民税・新契約（上限2.8万円）
pub const LIFE_NEW_RT: LifeInsuranceTier = LifeInsuranceTier { full_until: 12_000, half_until: 32_000, quarter_until: 56_000 };
/// 住民税・旧契約（上限3.5万円）
pub const LIFE_OLD_RT: LifeInsuranceTier = LifeInsuranceTier { full_until: 15_000, half_until: 40_000, quarter_until: 70_000 };
/// 生命保険料控除の合計の上限（一般・介護医療・個人年金の合計）
pub const LIFE_TOTAL_CAP_IT: i64 = 120_000;
pub const LIFE_TOTAL_CAP_RT: i64 = 70_000;

/// 地震保険料控除（所法77条、地方税法34条1項5号の3）
pub struct EarthquakeTable {
    /// 地震保険料を割る数（所得税は全額＝1、住民税は1/2＝2）
    pub premium_div: i64,
    /// 地震保険料分の上限、かつ旧長期損害保険料との合計の上限
    pub cap: i64,
    /// 旧長期損害保険料：この額以下は全額
    pub old_full_until: i64,
    /// 旧長期損害保険料：この額以下は 1/2 + 定額、超えると上限
    pub old_half_until: i64,
}
pub const EARTHQUAKE_IT: EarthquakeTable = EarthquakeTable { premium_div: 1, cap: 50_000, old_full_until: 10_000, old_half_until: 20_000 };
pub const EARTHQUAKE_RT: EarthquakeTable = EarthquakeTable { premium_div: 2, cap: 25_000, old_full_until: 5_000, old_half_until: 15_000 };

/// 医療費控除の足切り：10万円と総所得金額等の5%の少ない方（所法73条）
pub const MEDICAL_THRESHOLD: i64 = 100_000;
pub const MEDICAL_THRESHOLD_INCOME_PCT: i64 = 5;
/// 医療費控除の上限
pub const MEDICAL_CAP: i64 = 2_000_000;
/// セルフメディケーション税制の足切りと上限（措法41条の17）
pub const SELF_MEDICATION_THRESHOLD: i64 = 12_000;
pub const SELF_MEDICATION_CAP: i64 = 88_000;

// ───────────── 寄附金・確定申告 ─────────────

/// 寄附金控除・寄附金税額控除の自己負担額
pub const DONATION_SELF_BURDEN: i64 = 2_000;
/// 所得税の寄附金控除の対象額の上限：総所得金額等の40%（所法78条）
pub const DONATION_IT_INCOME_LIMIT_PCT: i64 = 40;
/// 住民税の寄附金税額控除の対象額の上限：総所得金額等の30%（地方税法37条の2・314条の7）
pub const DONATION_RT_INCOME_LIMIT_PCT: i64 = 30;
/// ふるさと納税の特例控除額の上限：調整控除後の所得割額の20%
pub const FURUSATO_SPECIAL_CAP_PCT: i64 = 20;
/// ワンストップ特例を使える寄附先の数の上限
pub const ONE_STOP_MAX_DONEES: usize = 5;
/// ふるさと納税の上限額を探すときの刻み
pub const FURUSATO_SEARCH_STEP: i64 = 1_000;
/// 給与収入がこの額を超えると年末調整の対象外で、確定申告が必要（所法190条）
pub const SALARY_FILING_REQUIRED_OVER: i64 = 20_000_000;

// ───────────── 税率 ─────────────

/// 申告分離課税の上場株式等の所得税率（%。措法8条の4・37条の11）
pub const STOCK_SEPARATE_IT_RATE_PCT: i64 = 15;
/// 住民税所得割の標準税率（%）[市町村, 道府県]（地方税法314条の3・35条）
pub const RESIDENT_RATE_PCT: [i64; 2] = [6, 4];
/// 申告分離課税の上場株式等の住民税率（%）[市町村, 道府県]（地方税法附則33条の2・35条の2の2）
pub const STOCK_SEPARATE_RT_RATE_PCT: [i64; 2] = [3, 2];

/// 調整控除（地方税法37条・314条の6）：合計所得金額がこの額を超えると適用なし
pub const ADJUSTMENT_CREDIT_INCOME_LIMIT: i64 = 25_000_000;
/// 調整控除：合計課税所得金額がこの額以下かどうかで算式が変わる
pub const ADJUSTMENT_CREDIT_THRESHOLD: i64 = 2_000_000;
/// 調整控除の率（%）[市町村, 道府県]
pub const ADJUSTMENT_CREDIT_RATE_PCT: [i64; 2] = [3, 2];
/// 調整控除の最低額 [市町村, 道府県]（合計2,500円。200万円超の算式で下回る場合）
pub const ADJUSTMENT_CREDIT_MIN: [i64; 2] = [1_500, 1_000];

// ───────────── 標準報酬・賞与 ─────────────

/// 健康保険の標準報酬月額表（等級ごとの「標準報酬月額」と「報酬月額の下限」。健康保険法40条）
pub static HEALTH_GRADES: [(i64, i64); 50] = [
    (58_000, 0),
    (68_000, 63_000),
    (78_000, 73_000),
    (88_000, 83_000),
    (98_000, 93_000),
    (104_000, 101_000),
    (110_000, 107_000),
    (118_000, 114_000),
    (126_000, 122_000),
    (134_000, 130_000),
    (142_000, 138_000),
    (150_000, 146_000),
    (160_000, 155_000),
    (170_000, 165_000),
    (180_000, 175_000),
    (190_000, 185_000),
    (200_000, 195_000),
    (220_000, 210_000),
    (240_000, 230_000),
    (260_000, 250_000),
    (280_000, 270_000),
    (300_000, 290_000),
    (320_000, 310_000),
    (340_000, 330_000),
    (360_000, 350_000),
    (380_000, 370_000),
    (410_000, 395_000),
    (440_000, 425_000),
    (470_000, 455_000),
    (500_000, 485_000),
    (530_000, 515_000),
    (560_000, 545_000),
    (590_000, 575_000),
    (620_000, 605_000),
    (650_000, 635_000),
    (680_000, 665_000),
    (710_000, 695_000),
    (750_000, 730_000),
    (790_000, 770_000),
    (830_000, 810_000),
    (880_000, 855_000),
    (930_000, 905_000),
    (980_000, 955_000),
    (1_030_000, 1_005_000),
    (1_090_000, 1_055_000),
    (1_150_000, 1_115_000),
    (1_210_000, 1_175_000),
    (1_270_000, 1_235_000),
    (1_330_000, 1_295_000),
    (1_390_000, 1_355_000),
];
/// 標準賞与額は賞与の1,000円未満を切り捨てた額
pub const BONUS_STANDARD_UNIT: i64 = 1_000;
/// 雇用保険料率（1/10000単位）の分母
pub const EMPLOYMENT_RATE_DENOM: i64 = 10_000;
/// 都道府県が表にないときに使う協会けんぽの料率の都道府県
pub const KYOKAI_KENPO_FALLBACK_PREFECTURE: &str = "東京都";

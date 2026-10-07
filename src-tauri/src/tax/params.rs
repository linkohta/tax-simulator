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
pub const DIFF_DISABLED: i64 = 10_000;
pub const DIFF_SPECIAL_DISABLED: i64 = 100_000;

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

/// 配当控除（1/1000 %単位）[課税総所得等1,000万円以下の部分, 超える部分]
pub const DIVIDEND_CREDIT_IT: [i64; 2] = [10_000, 5_000];
pub const DIVIDEND_CREDIT_RT_MUNI: [i64; 2] = [1_600, 800];
pub const DIVIDEND_CREDIT_RT_PREF: [i64; 2] = [1_200, 600];

// ───────────── 住民税 ─────────────

/// 均等割（市町村3,000＋道府県1,000＋森林環境税1,000）
pub const PER_CAPITA_TOTAL: i64 = 5_000;
/// 住宅ローン控除の住民税控除上限
pub const HOUSING_LOAN_RT_CAP: i64 = 97_500;
pub const HOUSING_LOAN_RT_RATE_PCT: i64 = 5;
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

/// 介護保険料率（40〜64歳、労使合計）
pub const CARE_RATE: i64 = 1_620;
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

//! 人的控除（基礎控除、配偶者・扶養親族、障害者・寡婦・ひとり親・勤労学生）と年齢・所得の区分

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


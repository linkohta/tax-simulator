//! 住民税（均等割、非課税限度額、ふるさと納税の特例控除割合、特別徴収）

// ───────────── 住民税 ─────────────

/// 均等割（市町村3,000＋道府県1,000＋森林環境税1,000）
pub const PER_CAPITA_TOTAL: i64 = 5_000;

/// 住民税の特別徴収（給与天引き）の回数：6月から翌年5月までの12回（地方税法321条の5）
pub const RESIDENT_INSTALLMENTS: i64 = 12;

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
pub static FURUSATO_BAND_NEGATIVE: FurusatoBand =
    FurusatoBand { upper: 0, special_x1000: 90_000, filing_ratio: (0, 1) };
/// 課税総所得がなく分離課税所得のみの場合（90% − 15.315%）
pub static FURUSATO_BAND_SEPARATE: FurusatoBand =
    FurusatoBand { upper: 0, special_x1000: 74_685, filing_ratio: (15_315, 74_685) };

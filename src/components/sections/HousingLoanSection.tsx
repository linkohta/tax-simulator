import { Check, MoneyInput, NumberInput, Section, Select } from "../Fields";
import { HousingLoanInput, HousingLoanMode, HousingType } from "../../types";

type Props = {
  housingLoan: HousingLoanInput;
  setHl: (patch: Partial<HousingLoanInput>) => void;
};

const MODE_OPTIONS: { value: HousingLoanMode; label: string }[] = [
  { value: "manual", label: "控除額を入力する（年末調整の証明書などで計算済みの額）" },
  { value: "auto", label: "年末残高から計算する" },
];

const TYPE_OPTIONS: { value: HousingType; label: string }[] = [
  { value: "certified", label: "認定長期優良住宅・認定低炭素住宅" },
  { value: "zeh", label: "ZEH水準省エネ住宅" },
  { value: "energySaving", label: "省エネ基準適合住宅" },
  { value: "other", label: "その他の住宅" },
];

/** 入力項目の出し分けに使う居住開始年の区切り（税額の計算は Rust 側） */
const SPECIFIC_ACQUISITION_UNTIL = 2021;
const CHILD_REARING_FROM = 2024;

export function HousingLoanSection({ housingLoan: hl, setHl }: Props) {
  const auto = hl.mode === "auto";
  const badge = auto ? (hl.yearEndBalance > 0 ? "自動計算" : undefined) : hl.creditAmount > 0 ? "手入力" : undefined;

  return (
    <Section title="住宅ローン控除" badge={badge}>
      <Select label="入力方法" value={hl.mode} onChange={(mode) => setHl({ mode })} options={MODE_OPTIONS} />
      {auto ? (
        <>
          <div className="grid">
            <MoneyInput
              label="住宅借入金等の年末残高"
              value={hl.yearEndBalance}
              onChange={(yearEndBalance) => setHl({ yearEndBalance })}
              hint="自分の持分・居住用部分に対応する額。借入限度額を超える部分は自動で切り捨てます"
            />
            <NumberInput
              label="居住を開始した年"
              value={hl.moveInYear}
              onChange={(moveInYear) => setHl({ moveInYear })}
              min={2000}
              max={2026}
              suffix="年"
              hint="西暦。2017年より前は控除期間が終わっています"
            />
            <Select
              label="新築・中古"
              value={hl.existingHouse ? "existing" : "new"}
              onChange={(v) => setHl({ existingHouse: v === "existing" })}
              options={[
                { value: "new", label: "新築・買取再販住宅" },
                { value: "existing", label: "中古住宅" },
              ]}
            />
            <Select
              label="住宅の区分"
              value={hl.housingType}
              onChange={(housingType) => setHl({ housingType })}
              options={TYPE_OPTIONS}
              hint={
                hl.moveInYear <= SPECIFIC_ACQUISITION_UNTIL
                  ? "2021年以前の入居では、認定住宅（新築）かどうかだけが限度額に影響します"
                  : "借入限度額と控除期間が変わります"
              }
            />
          </div>
          {hl.moveInYear >= CHILD_REARING_FROM && (
            <Check
              label="子育て世帯・若者夫婦世帯に当たる"
              checked={hl.childRearing}
              onChange={(childRearing) => setHl({ childRearing })}
              hint="居住を開始した年の年末に、19歳未満の扶養親族がいる、または夫婦のどちらかが40歳未満。借入限度額が上乗せされます"
            />
          )}
          {hl.moveInYear <= SPECIFIC_ACQUISITION_UNTIL && (
            <Check
              label="消費税率8%・10%で取得した（特定取得）"
              checked={hl.specificAcquisition}
              onChange={(specificAcquisition) => setHl({ specificAcquisition })}
              hint="借入限度額が4,000万円（認定住宅5,000万円）、住民税から控除できる限度額が7%（最高136,500円）になります"
            />
          )}
          <p className="note">
            控除額は、年末残高（借入限度額まで）× 控除率（2022年以後の入居は0.7%、2021年以前は1%）です。所得税で引ききれない分は住民税から控除します。
          </p>
        </>
      ) : (
        <>
          <MoneyInput
            label="住宅借入金等特別控除の額"
            value={hl.creditAmount}
            onChange={(creditAmount) => setHl({ creditAmount })}
            hint="年末残高×控除率（0.7%等）で計算済みの控除可能額。所得税で引ききれない分は住民税から控除"
          />
          {hl.creditAmount > 0 && (
            <Check
              label="2026年以後に居住を開始した"
              checked={hl.movedIn2026OrLater}
              onChange={(movedIn2026OrLater) => setHl({ movedIn2026OrLater })}
              hint="住民税から控除できる限度額の計算方法が異なります"
            />
          )}
          {hl.creditAmount > 0 && !hl.movedIn2026OrLater && (
            <Check
              label="2014年4月〜2021年12月に居住を開始し、消費税率8%・10%で取得した"
              checked={hl.specificAcquisition}
              onChange={(specificAcquisition) => setHl({ specificAcquisition })}
              hint="住民税から控除できる限度額が7%（最高136,500円）になります。それ以外は5%（最高97,500円）"
            />
          )}
        </>
      )}
    </Section>
  );
}

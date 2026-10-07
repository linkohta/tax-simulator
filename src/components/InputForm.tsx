import {
  Check,
  DISABILITY_OPTIONS,
  MoneyInput,
  NumberInput,
  Section,
  Select,
  yen,
} from "./Fields";
import { DependentInput, TaxInput, newDependent } from "../types";

type Props = {
  input: TaxInput;
  setInput: (updater: (prev: TaxInput) => TaxInput) => void;
  prefectures: string[];
};

/** ネストしたオブジェクトの1セクションを部分更新するヘルパー */
function useSection<K extends keyof TaxInput>(
  setInput: Props["setInput"],
  key: K,
) {
  return (patch: Partial<TaxInput[K]>) =>
    setInput((prev) => ({ ...prev, [key]: { ...(prev[key] as object), ...patch } }));
}

export function InputForm({ input, setInput, prefectures }: Props) {
  const setSalary = useSection(setInput, "salary");
  const setSi = useSection(setInput, "socialInsurance");
  const setFamily = useSection(setInput, "family");
  const setIns = useSection(setInput, "insurance");
  const setPs = useSection(setInput, "pensionSavings");
  const setMed = useSection(setInput, "medical");
  const setDon = useSection(setInput, "donations");
  const setStock = useSection(setInput, "stocks");
  const setHl = useSection(setInput, "housingLoan");

  const si = input.socialInsurance;
  const fam = input.family;
  const ins = input.insurance;
  const don = input.donations;
  const st = input.stocks;
  const med = input.medical;

  const updateDependent = (i: number, patch: Partial<DependentInput>) =>
    setFamily({
      dependents: fam.dependents.map((d, j) => (j === i ? { ...d, ...patch } : d)),
    });

  const furusatoTotal = don.furusato.reduce((s, e) => s + e.amount, 0);
  const bonusTotal = si.bonuses.reduce((s, b) => s + b, 0);

  return (
    <div className="form">
      <Section title="基本情報・給与" defaultOpen>
        <div className="grid">
          <NumberInput
            label="本人の年齢"
            value={input.age}
            onChange={(age) => setInput((p) => ({ ...p, age }))}
            suffix="歳"
            hint="2026年12月31日時点"
          />
          <MoneyInput
            label="給与収入（年額・賞与込み）"
            value={input.salary.annualIncome}
            onChange={(annualIncome) => setSalary({ annualIncome })}
            hint="源泉徴収票の「支払金額」"
          />
        </div>
      </Section>

      <Section title="社会保険料" badge={si.mode === "auto" ? "概算" : "手入力"} defaultOpen>
        <Select
          label="計算方法"
          value={si.mode}
          onChange={(mode) => setSi({ mode })}
          options={[
            { value: "auto", label: "給与から概算する（協会けんぽ・厚生年金）" },
            { value: "manual", label: "金額を入力する（源泉徴収票の社会保険料等の金額）" },
          ]}
        />
        {si.mode === "manual" ? (
          <MoneyInput
            label="給与天引きの社会保険料（年額）"
            value={si.manualAmount}
            onChange={(manualAmount) => setSi({ manualAmount })}
          />
        ) : (
          <>
            <div className="grid">
              <Select
                label="協会けんぽ 都道府県"
                value={si.prefecture}
                onChange={(prefecture) => setSi({ prefecture })}
                options={prefectures.map((p) => ({ value: p, label: p }))}
              />
              <MoneyInput
                label="月額給与（報酬月額）"
                value={si.monthlySalary}
                onChange={(monthlySalary) => setSi({ monthlySalary })}
                hint="基本給＋残業代・諸手当＋通勤手当"
              />
            </div>
            <div className="list">
              <div className="list-head">
                <span>賞与（1回ごと）</span>
                <button
                  type="button"
                  className="small"
                  onClick={() => setSi({ bonuses: [...si.bonuses, 0] })}
                >
                  ＋ 追加
                </button>
              </div>
              {si.bonuses.map((b, i) => (
                <div className="row" key={i}>
                  <MoneyInput
                    label={`賞与 ${i + 1}回目`}
                    value={b}
                    onChange={(v) =>
                      setSi({ bonuses: si.bonuses.map((x, j) => (j === i ? v : x)) })
                    }
                  />
                  <button
                    type="button"
                    className="small danger"
                    onClick={() => setSi({ bonuses: si.bonuses.filter((_, j) => j !== i) })}
                  >
                    削除
                  </button>
                </div>
              ))}
            </div>
            <p className="note">
              月額×12＋賞与 = {yen(si.monthlySalary * 12 + bonusTotal)}
              {si.monthlySalary * 12 + bonusTotal !== input.salary.annualIncome && (
                <>
                  {" "}
                  <button
                    type="button"
                    className="link"
                    onClick={() => setSalary({ annualIncome: si.monthlySalary * 12 + bonusTotal })}
                  >
                    給与収入に反映
                  </button>
                </>
              )}
            </p>
            <div className="grid">
              <NumberInput
                label="子ども・子育て支援金の負担月数"
                value={si.childcareSupportMonths}
                onChange={(childcareSupportMonths) => setSi({ childcareSupportMonths })}
                max={12}
                suffix="か月"
                hint="2026年4月分から。翌月徴収なら年内の天引きは8か月"
              />
              <Check
                label="雇用保険に加入している"
                checked={si.employmentInsurance}
                onChange={(employmentInsurance) => setSi({ employmentInsurance })}
              />
            </div>
          </>
        )}
        <MoneyInput
          label="その他に支払った社会保険料"
          value={si.otherAmount}
          onChange={(otherAmount) => setSi({ otherAmount })}
          hint="国民年金・国民健康保険（家族分含む）など給与天引き以外"
        />
      </Section>

      <Section
        title="配偶者・扶養親族"
        badge={
          [fam.hasSpouse ? "配偶者" : "", fam.dependents.length ? `扶養${fam.dependents.length}人` : ""]
            .filter(Boolean)
            .join("・") || undefined
        }
      >
        <Check
          label="配偶者がいる"
          checked={fam.hasSpouse}
          onChange={(hasSpouse) => setFamily({ hasSpouse })}
        />
        {fam.hasSpouse && (
          <div className="card">
            <div className="grid">
              <NumberInput
                label="配偶者の年齢"
                value={fam.spouse.age}
                onChange={(age) => setFamily({ spouse: { ...fam.spouse, age } })}
                suffix="歳"
              />
              <MoneyInput
                label="配偶者の給与収入"
                value={fam.spouse.salaryIncome}
                onChange={(salaryIncome) => setFamily({ spouse: { ...fam.spouse, salaryIncome } })}
              />
              <MoneyInput
                label="配偶者の給与以外の所得"
                value={fam.spouse.otherIncome}
                onChange={(otherIncome) => setFamily({ spouse: { ...fam.spouse, otherIncome } })}
                hint="事業・年金・申告した配当など（所得金額）"
              />
              <Select
                label="配偶者の障害"
                value={fam.spouse.disability}
                onChange={(disability) => setFamily({ spouse: { ...fam.spouse, disability } })}
                options={DISABILITY_OPTIONS}
              />
            </div>
          </div>
        )}

        <div className="list">
          <div className="list-head">
            <span>扶養親族（16歳未満の子も障害者控除・判定のため入力可）</span>
            <button
              type="button"
              className="small"
              onClick={() => setFamily({ dependents: [...fam.dependents, newDependent()] })}
            >
              ＋ 追加
            </button>
          </div>
          {fam.dependents.map((d, i) => (
            <div className="card" key={i}>
              <div className="card-head">
                <input
                  className="name"
                  placeholder={`親族 ${i + 1}`}
                  value={d.name}
                  onChange={(e) => updateDependent(i, { name: e.target.value })}
                />
                <button
                  type="button"
                  className="small danger"
                  onClick={() =>
                    setFamily({ dependents: fam.dependents.filter((_, j) => j !== i) })
                  }
                >
                  削除
                </button>
              </div>
              <div className="grid">
                <NumberInput
                  label="年齢"
                  value={d.age}
                  onChange={(age) => updateDependent(i, { age })}
                  suffix="歳"
                  hint="12/31時点"
                />
                <MoneyInput
                  label="給与収入"
                  value={d.salaryIncome}
                  onChange={(salaryIncome) => updateDependent(i, { salaryIncome })}
                />
                <MoneyInput
                  label="給与以外の所得"
                  value={d.otherIncome}
                  onChange={(otherIncome) => updateDependent(i, { otherIncome })}
                />
                <Select
                  label="障害"
                  value={d.disability}
                  onChange={(disability) => updateDependent(i, { disability })}
                  options={DISABILITY_OPTIONS}
                />
              </div>
              {d.age >= 70 && (
                <Check
                  label="同居の直系尊属（同居老親等）"
                  checked={d.cohabitingParent}
                  onChange={(cohabitingParent) => updateDependent(i, { cohabitingParent })}
                />
              )}
            </div>
          ))}
        </div>

        <div className="grid">
          <Select
            label="本人の障害"
            value={fam.selfDisability === "specialCohabiting" ? "special" : fam.selfDisability}
            onChange={(selfDisability) => setFamily({ selfDisability })}
            options={DISABILITY_OPTIONS.slice(0, 3)}
          />
          <Select
            label="寡婦・ひとり親"
            value={fam.widowStatus}
            onChange={(widowStatus) => setFamily({ widowStatus })}
            options={[
              { value: "none", label: "該当なし" },
              { value: "widow", label: "寡婦" },
              { value: "singleMother", label: "ひとり親（母）" },
              { value: "singleFather", label: "ひとり親（父）" },
            ]}
          />
        </div>
        <Check
          label="勤労学生"
          checked={fam.workingStudent}
          onChange={(workingStudent) => setFamily({ workingStudent })}
        />
      </Section>

      <Section title="生命保険料・地震保険料">
        <h4>生命保険料控除（年間支払保険料）</h4>
        <div className="grid">
          <MoneyInput label="一般生命保険料（新契約）" value={ins.lifeNew} onChange={(lifeNew) => setIns({ lifeNew })} hint="2012年1月1日以後の契約" />
          <MoneyInput label="一般生命保険料（旧契約）" value={ins.lifeOld} onChange={(lifeOld) => setIns({ lifeOld })} hint="2011年12月31日以前の契約" />
          <MoneyInput label="介護医療保険料" value={ins.medicalCare} onChange={(medicalCare) => setIns({ medicalCare })} />
          <MoneyInput label="個人年金保険料（新契約）" value={ins.pensionNew} onChange={(pensionNew) => setIns({ pensionNew })} />
          <MoneyInput label="個人年金保険料（旧契約）" value={ins.pensionOld} onChange={(pensionOld) => setIns({ pensionOld })} />
        </div>
        <h4>地震保険料控除</h4>
        <div className="grid">
          <MoneyInput label="地震保険料" value={ins.earthquake} onChange={(earthquake) => setIns({ earthquake })} />
          <MoneyInput label="旧長期損害保険料" value={ins.oldLongTerm} onChange={(oldLongTerm) => setIns({ oldLongTerm })} hint="2006年末以前契約の長期損害保険" />
        </div>
      </Section>

      <Section title="iDeCo・小規模企業共済">
        <div className="grid">
          <MoneyInput
            label="iDeCo 掛金（年額）"
            value={input.pensionSavings.ideco}
            onChange={(ideco) => setPs({ ideco })}
            hint={`月額 ${yen(Math.round(input.pensionSavings.ideco / 12))}`}
          />
          <MoneyInput
            label="企業型DC マッチング拠出（年額）"
            value={input.pensionSavings.corporateDcMatching}
            onChange={(corporateDcMatching) => setPs({ corporateDcMatching })}
          />
          <MoneyInput
            label="小規模企業共済 掛金（年額）"
            value={input.pensionSavings.smallBusinessMutualAid}
            onChange={(smallBusinessMutualAid) => setPs({ smallBusinessMutualAid })}
          />
        </div>
      </Section>

      <Section title="寄附金（ふるさと納税等）" badge={furusatoTotal ? yen(furusatoTotal) : undefined}>
        <div className="list">
          <div className="list-head">
            <span>ふるさと納税（寄附先ごと）</span>
            <button
              type="button"
              className="small"
              onClick={() => setDon({ furusato: [...don.furusato, { name: "", amount: 0 }] })}
            >
              ＋ 追加
            </button>
          </div>
          {don.furusato.map((f, i) => (
            <div className="row" key={i}>
              <input
                className="name"
                placeholder="自治体名"
                value={f.name}
                onChange={(e) =>
                  setDon({
                    furusato: don.furusato.map((x, j) => (j === i ? { ...x, name: e.target.value } : x)),
                  })
                }
              />
              <MoneyInput
                label=""
                value={f.amount}
                onChange={(amount) =>
                  setDon({ furusato: don.furusato.map((x, j) => (j === i ? { ...x, amount } : x)) })
                }
              />
              <button
                type="button"
                className="small danger"
                onClick={() => setDon({ furusato: don.furusato.filter((_, j) => j !== i) })}
              >
                削除
              </button>
            </div>
          ))}
          {don.furusato.length > 0 && <p className="note">合計 {yen(furusatoTotal)}（{don.furusato.length}団体）</p>}
        </div>
        <Check
          label="ワンストップ特例を利用する"
          checked={don.oneStop}
          onChange={(oneStop) => setDon({ oneStop })}
          hint="確定申告不要な給与所得者で寄附先5団体以内の場合のみ有効"
        />
        <h4>その他の寄附金</h4>
        <div className="grid">
          <MoneyInput
            label="所得税のみ控除対象の寄附"
            value={don.otherIncomeTaxOnly}
            onChange={(otherIncomeTaxOnly) => setDon({ otherIncomeTaxOnly })}
            hint="国・公益法人・認定NPO等（所得控除として計算）"
          />
          <MoneyInput
            label="住民税の条例指定寄附にも該当する寄附"
            value={don.otherResidentDesignated}
            onChange={(otherResidentDesignated) => setDon({ otherResidentDesignated })}
            hint="都道府県・市区町村の両方が指定（共同募金・日赤等）"
          />
        </div>
      </Section>

      <Section title="医療費・住宅ローン控除">
        <Select
          label="医療費の控除方式"
          value={med.mode}
          onChange={(mode) => setMed({ mode })}
          options={[
            { value: "normal", label: "医療費控除（通常）" },
            { value: "selfMedication", label: "セルフメディケーション税制" },
          ]}
        />
        {med.mode === "normal" ? (
          <div className="grid">
            <MoneyInput label="支払った医療費" value={med.expenses} onChange={(expenses) => setMed({ expenses })} hint="生計を一にする家族分を含む" />
            <MoneyInput label="保険金などで補てんされる金額" value={med.reimbursed} onChange={(reimbursed) => setMed({ reimbursed })} />
          </div>
        ) : (
          <MoneyInput label="特定一般用医薬品等の購入費" value={med.otcPurchases} onChange={(otcPurchases) => setMed({ otcPurchases })} />
        )}
        <MoneyInput
          label="住宅借入金等特別控除の額"
          value={input.housingLoan.creditAmount}
          onChange={(creditAmount) => setHl({ creditAmount })}
          hint="年末残高×控除率（0.7%等）で計算済みの控除可能額。所得税で引ききれない分は住民税から控除"
        />
        {input.housingLoan.creditAmount > 0 && (
          <Check
            label="2026年以後に居住を開始した"
            checked={input.housingLoan.movedIn2026OrLater}
            onChange={(movedIn2026OrLater) => setHl({ movedIn2026OrLater })}
            hint="住民税から控除できる限度額の計算方法が異なります"
          />
        )}
      </Section>

      <Section title="株式（上場株式等）">
        <div className="grid">
          <MoneyInput
            label="配当等（税引前・年額）"
            value={st.dividends}
            onChange={(dividends) => setStock({ dividends })}
            hint="NISA口座分は含めない"
          />
          <Select
            label="配当の課税方式"
            value={st.dividendMode}
            onChange={(dividendMode) => setStock({ dividendMode })}
            options={[
              { value: "noDeclare", label: "申告不要（源泉徴収で完結）" },
              { value: "comprehensive", label: "総合課税（配当控除）" },
              { value: "separate", label: "申告分離課税" },
            ]}
            hint="所得税と住民税で同じ方式になります"
          />
          <MoneyInput
            label="譲渡損益（年間通算・税引前）"
            value={st.capitalGain}
            onChange={(capitalGain) => setStock({ capitalGain })}
            allowNegative
            hint="損失はマイナスで入力"
          />
          <MoneyInput
            label="繰越損失（前年以前分）"
            value={st.lossCarryforward}
            onChange={(lossCarryforward) => setStock({ lossCarryforward })}
          />
        </div>
        <Check
          label="譲渡損益を確定申告する"
          checked={st.declareCapitalGain}
          onChange={(declareCapitalGain) => setStock({ declareCapitalGain })}
          hint="源泉徴収ありの特定口座で申告しない場合はチェック不要。損益通算・繰越控除には申告が必要"
        />
      </Section>
    </div>
  );
}

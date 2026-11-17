import { Check, MoneyInput, Section, Select, yen } from "../Fields";
import { StockAccount, StockAccountKind, StockInput, newStockAccount } from "../../types";

type Props = {
  stocks: StockInput;
  setStock: (patch: Partial<StockInput>) => void;
};

const ACCOUNT_KIND_OPTIONS: { value: StockAccountKind; label: string }[] = [
  { value: "withholding", label: "特定口座（源泉徴収あり）" },
  { value: "noWithholding", label: "特定口座（源泉徴収なし）" },
  { value: "general", label: "一般口座" },
];

export function StockSection({ stocks: st, setStock }: Props) {
  const updateAccount = (i: number, patch: Partial<StockAccount>) =>
    setStock({ accounts: st.accounts.map((a, j) => (j === i ? { ...a, ...patch } : a)) });

  // 表示用の合計（税の計算は Rust 側）
  const dividendTotal = st.dividends + st.dividendsFund + st.dividendsForeignFund + st.dividendsNoCredit;
  const gainTotal = st.accounts.reduce((s, a) => s + a.capitalGain, 0);
  const badge =
    [dividendTotal ? `配当 ${yen(dividendTotal)}` : "", st.accounts.length ? `${st.accounts.length}口座` : ""]
      .filter(Boolean)
      .join("・") || undefined;

  return (
    <Section title="株式（上場株式等）" badge={badge}>
      <h4>配当等（税引前・年額。NISA口座分は含めない）</h4>
      <div className="grid">
        <MoneyInput
          label="国内株式の配当など"
          value={st.dividends}
          onChange={(dividends) => setStock({ dividends })}
          hint="剰余金の配当等。国内株式ETFを含む（配当控除10%）"
        />
        <MoneyInput
          label="投資信託の分配金（国内株式中心）"
          value={st.dividendsFund}
          onChange={(dividendsFund) => setStock({ dividendsFund })}
          hint="外貨建資産・株式以外の割合が50%以下（配当控除5%）"
        />
        <MoneyInput
          label="投資信託の分配金（外貨建資産などを含む）"
          value={st.dividendsForeignFund}
          onChange={(dividendsForeignFund) => setStock({ dividendsForeignFund })}
          hint="同割合が50%超75%以下（配当控除2.5%）"
        />
        <MoneyInput
          label="配当控除の対象外の配当等"
          value={st.dividendsNoCredit}
          onChange={(dividendsNoCredit) => setStock({ dividendsNoCredit })}
          hint="外国株式、J-REIT、同割合75%超の投資信託など"
        />
      </div>
      <Select
        label="配当の課税方式"
        value={st.dividendMode}
        onChange={(dividendMode) => setStock({ dividendMode })}
        options={[
          { value: "noDeclare", label: "申告不要（源泉徴収で完結）" },
          { value: "comprehensive", label: "総合課税（配当控除）" },
          { value: "separate", label: "申告分離課税" },
        ]}
        hint="所得税と住民税で同じ方式になります。配当控除の率は総合課税のときだけ使います。結果の「株式の課税方式の比較」で税額を見比べられます"
      />

      <div className="list">
        <div className="list-head">
          <span>譲渡損益（口座ごと・年間通算・税引前）</span>
          <button
            type="button"
            className="small"
            onClick={() => setStock({ accounts: [...st.accounts, newStockAccount()] })}
          >
            ＋ 口座を追加
          </button>
        </div>
        {st.accounts.map((a, i) => (
          <div className="card" key={i}>
            <div className="card-head">
              <input
                className="name"
                placeholder={`口座 ${i + 1}（証券会社名など）`}
                value={a.name}
                onChange={(e) => updateAccount(i, { name: e.target.value })}
              />
              <button
                type="button"
                className="small danger"
                onClick={() => setStock({ accounts: st.accounts.filter((_, j) => j !== i) })}
              >
                削除
              </button>
            </div>
            <div className="grid">
              <Select
                label="口座の種類"
                value={a.kind}
                onChange={(kind) => updateAccount(i, { kind })}
                options={ACCOUNT_KIND_OPTIONS}
              />
              <MoneyInput
                label="譲渡損益"
                value={a.capitalGain}
                onChange={(capitalGain) => updateAccount(i, { capitalGain })}
                allowNegative
                hint="損失はマイナスで入力"
              />
            </div>
            {a.kind === "withholding" ? (
              <Check
                label="この口座の損益を確定申告する"
                checked={a.declare}
                onChange={(declare) => updateAccount(i, { declare })}
                hint="申告しなければ源泉徴収で完結。ほかの口座や配当との損益通算、損失の繰越には申告が必要"
              />
            ) : (
              <p className="note">この種類の口座は、損益があれば確定申告するものとして計算します。</p>
            )}
          </div>
        ))}
        {st.accounts.length > 1 && <p className="note">損益の合計 {yen(gainTotal)}（{st.accounts.length}口座）</p>}
      </div>

      <MoneyInput
        label="繰越損失（前年以前分）"
        value={st.lossCarryforward}
        onChange={(lossCarryforward) => setStock({ lossCarryforward })}
        hint="前年以前の確定申告で繰り越した上場株式等の譲渡損失"
      />

      <h4>外国税額控除（配当を申告する場合）</h4>
      <div className="grid">
        <MoneyInput
          label="国外所得金額"
          value={st.foreignIncome}
          onChange={(foreignIncome) => setStock({ foreignIncome })}
          hint="外国株式の配当など（外国所得税を引く前の額）。上の配当等にも含めて入力"
        />
        <MoneyInput
          label="外国所得税額"
          value={st.foreignTaxPaid}
          onChange={(foreignTaxPaid) => setStock({ foreignTaxPaid })}
          hint="現地で源泉徴収された税額（円換算）"
        />
      </div>
    </Section>
  );
}

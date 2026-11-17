import { MoneyInput, Section, yen } from "../Fields";
import { MiscIncomeInput } from "../../types";

type Props = {
  miscIncome: MiscIncomeInput;
  setMisc: (patch: Partial<MiscIncomeInput>) => void;
};

export function MiscIncomeSection({ miscIncome: m, setMisc }: Props) {
  const badge =
    [m.pensionIncome ? `年金 ${yen(m.pensionIncome)}` : "", m.sideRevenue ? `副業 ${yen(m.sideRevenue)}` : ""]
      .filter(Boolean)
      .join("・") || undefined;

  return (
    <Section title="公的年金・副業（雑所得）" badge={badge}>
      <MoneyInput
        label="公的年金等の収入金額（年額）"
        value={m.pensionIncome}
        onChange={(pensionIncome) => setMisc({ pensionIncome })}
        hint="源泉徴収票の「支払金額」。老齢年金、企業年金など。遺族年金・障害年金は非課税なので含めない"
      />
      <h4>副業などの雑所得</h4>
      <div className="grid">
        <MoneyInput
          label="収入（年額）"
          value={m.sideRevenue}
          onChange={(sideRevenue) => setMisc({ sideRevenue })}
          hint="原稿料、業務委託の報酬、フリマ・暗号資産の売却益など"
        />
        <MoneyInput
          label="必要経費"
          value={m.sideExpenses}
          onChange={(sideExpenses) => setMisc({ sideExpenses })}
          hint="その収入を得るために直接かかった費用"
        />
      </div>
      <p className="note">
        雑所得の赤字は、給与などほかの所得からは引けません（公的年金等の雑所得とは通算します）。年金から天引きされた介護保険料などは「その他に支払った社会保険料」に入力してください。
      </p>
    </Section>
  );
}

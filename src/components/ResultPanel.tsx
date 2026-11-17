import { DeductionItem, DividendMode, StockOption, TaxResult } from "../types";
import { yen } from "./Fields";

const pct = (n: number) => `${n.toFixed(2)}%`;

function Rows({ items }: { items: DeductionItem[] }) {
  return (
    <>
      {items.map((d, i) => (
        <tr key={`${i}:${d.label}`}>
          <td className="indent">{d.label}</td>
          <td className="num">{yen(d.amount)}</td>
        </tr>
      ))}
    </>
  );
}

function Row({ label, value, strong, minus }: { label: string; value: number; strong?: boolean; minus?: boolean }) {
  return (
    <tr className={strong ? "strong" : undefined}>
      <td>{label}</td>
      <td className="num">{minus && value > 0 ? "△ " : ""}{yen(value)}</td>
    </tr>
  );
}

const DIVIDEND_MODE_LABELS: Record<DividendMode, string> = {
  noDeclare: "配当：申告不要",
  comprehensive: "配当：総合課税",
  separate: "配当：申告分離",
};

const signedYen = (n: number) => (n > 0 ? "+" : n === 0 ? "±" : "") + yen(n);

/** 株式の課税方式ごとの税額の比較表 */
function StockOptions({ options }: { options: StockOption[] }) {
  const hasDividends = options.some((o) => o.dividendMode !== options[0].dividendMode);
  return (
    <details open>
      <summary>株式の課税方式の比較</summary>
      <table className="compare">
        <thead>
          <tr>
            <th>方式</th>
            <th className="num">税額合計</th>
            <th className="num">現在との差</th>
          </tr>
        </thead>
        <tbody>
          {options.map((o, i) => (
            <tr key={i} className={o.isCurrent ? "current" : undefined}>
              <td>
                {[
                  hasDividends ? DIVIDEND_MODE_LABELS[o.dividendMode] : "",
                  o.declareWithholdingAccounts === null
                    ? ""
                    : o.declareWithholdingAccounts
                      ? "譲渡：申告する"
                      : "譲渡：申告しない",
                ]
                  .filter(Boolean)
                  .join("／")}
                {o.isBest && <span className="tag best">最も有利</span>}
                {o.isCurrent && <span className="tag">現在</span>}
                {o.lossCarriedToNextYear > 0 && (
                  <span className="field-hint">翌年に繰り越す損失 {yen(o.lossCarriedToNextYear)}</span>
                )}
              </td>
              <td className="num">{yen(o.totalTax)}</td>
              <td className="num">{signedYen(o.diffFromCurrent)}</td>
            </tr>
          ))}
        </tbody>
      </table>
      <p className="meta">
        税額合計は所得税・住民税・申告不要とした分の源泉徴収税の合計です。「譲渡」は源泉徴収ありの特定口座をすべて申告する／しない場合です。
        申告すると合計所得金額が増え、国民健康保険料や扶養の判定などに影響することがあります（この比較には含めていません）。
      </p>
    </details>
  );
}

export function ResultPanel({ result }: { result: TaxResult | null }) {
  if (!result) return <div className="result empty">計算中…</div>;
  const { summary: s, income: inc, incomeTax: it, residentTax: rt, socialInsurance: si, furusato: fu } = result;

  return (
    <div className="result">
      <div className="kpis">
        <div className="kpi">
          <span>所得税（復興税込）</span>
          <strong>{yen(s.incomeTax)}</strong>
        </div>
        <div className="kpi">
          <span>住民税（令和9年度）</span>
          <strong>{yen(s.residentTax)}</strong>
        </div>
        <div className="kpi">
          <span>社会保険料</span>
          <strong>{yen(s.socialInsurance)}</strong>
        </div>
        <div className="kpi accent">
          <span>手取り（概算）</span>
          <strong>{yen(s.takeHome)}</strong>
          <span>月平均 {yen(s.takeHomeMonthly)}</span>
        </div>
      </div>

      <div className="furusato">
        <div>
          <span className="label">ふるさと納税 上限額の目安</span>
          <strong>{yen(fu.limit)}</strong>
        </div>
        <div className="sub">
          現在の寄附 {yen(fu.currentTotal)} ／ 実質負担 {yen(fu.currentBurden)}
          {fu.currentTotal > 0 && (
            <>
              <br />
              軽減額：所得税 {yen(fu.incomeTaxReduction)}・住民税 {yen(fu.residentTaxReduction)}
            </>
          )}
        </div>
        {fu.limit > 0 && fu.currentTotal > fu.limit && (
          <div className="warn-inline">上限の目安を超えています（超過分は自己負担）</div>
        )}
      </div>

      {result.warnings.length > 0 && (
        <ul className="warnings">
          {result.warnings.map((w, i) => (
            <li key={`${i}:${w}`}>{w}</li>
          ))}
        </ul>
      )}

      <p className="meta">
        税負担率 {pct(s.effectiveTaxRate)}（所得税・住民税・株式源泉税 ÷ 収入）／ 所得税の限界税率 {it.marginalRate}%
        {s.filingRequired ? " ／ 確定申告が必要" : ""}
      </p>

      <details open>
        <summary>所得の計算</summary>
        <table>
          <tbody>
            <Row label="給与収入" value={inc.salaryIncome} />
            <Row label="給与所得控除" value={inc.salaryDeduction} minus />
            {inc.incomeAdjustmentDeduction > 0 && <Row label="所得金額調整控除" value={inc.incomeAdjustmentDeduction} minus />}
            <Row label="給与所得" value={inc.salaryNet} strong />
            {inc.pensionNet > 0 && <Row label="雑所得（公的年金等）" value={inc.pensionNet} />}
            {inc.sideNet !== 0 && <Row label="雑所得（副業など：収入 − 経費）" value={inc.sideNet} />}
            {inc.sideNet < 0 && <Row label="雑所得の金額（赤字はほかの所得から引けない）" value={inc.miscIncome} />}
            {inc.dividendComprehensive > 0 && <Row label="配当所得（総合課税）" value={inc.dividendComprehensive} />}
            {inc.dividendSeparate > 0 && <Row label="配当所得等（分離・通算後）" value={inc.dividendSeparate} />}
            {inc.capitalGainSeparate > 0 && <Row label="譲渡所得等（分離）" value={inc.capitalGainSeparate} />}
            <Row label="合計所得金額" value={inc.totalIncomeBeforeCarryover} strong />
            {inc.lossCarriedToNextYear > 0 && <Row label="翌年に繰り越す譲渡損失" value={inc.lossCarriedToNextYear} />}
          </tbody>
        </table>
      </details>

      {si.total > 0 && (
        <details>
          <summary>社会保険料{si.estimated ? "（概算）" : ""}</summary>
          <table>
            <tbody>
              {si.estimated && (
                <>
                  <Row label="健康保険" value={si.health} />
                  {si.care > 0 && <Row label="介護保険" value={si.care} />}
                  {si.childcareSupport > 0 && <Row label="子ども・子育て支援金" value={si.childcareSupport} />}
                  <Row label="厚生年金保険" value={si.pension} />
                  {si.employment > 0 && <Row label="雇用保険" value={si.employment} />}
                </>
              )}
              {si.other > 0 && <Row label="その他（国民年金等）" value={si.other} />}
              <Row label="合計" value={si.total} strong />
            </tbody>
          </table>
          {si.estimated && (
            <p className="meta">
              標準報酬月額：健保 {yen(si.standardMonthlyHealth)} ／ 厚年 {yen(si.standardMonthlyPension)}
            </p>
          )}
        </details>
      )}

      <details open>
        <summary>所得税（令和8年分）</summary>
        <table>
          <tbody>
            <tr className="group"><td colSpan={2}>所得控除</td></tr>
            <Rows items={it.deductions} />
            <Row label="所得控除 合計" value={it.totalDeductions} strong />
            <Row label="課税総所得金額" value={it.taxableOrdinary} />
            {it.taxableSeparate > 0 && <Row label="課税分離所得（株式等）" value={it.taxableSeparate} />}
            <Row label="算出税額（総合）" value={it.taxOnOrdinary} />
            {it.taxOnSeparate > 0 && <Row label="算出税額（分離）" value={it.taxOnSeparate} />}
            {it.credits.length > 0 && <tr className="group"><td colSpan={2}>税額控除</td></tr>}
            <Rows items={it.credits} />
            <Row label="基準所得税額" value={it.baseTax} />
            <Row label="復興特別所得税（2.1%）" value={it.reconstructionTax} />
            {it.foreignTaxCredit > 0 && <Row label="外国税額控除" value={it.foreignTaxCredit} minus />}
            <Row label="所得税及び復興特別所得税" value={it.total} strong />
          </tbody>
        </table>
      </details>

      <details open>
        <summary>住民税（令和9年度）</summary>
        <table>
          <tbody>
            <tr className="group"><td colSpan={2}>所得控除</td></tr>
            <Rows items={rt.deductions} />
            <Row label="所得控除 合計" value={rt.totalDeductions} strong />
            <Row label="課税総所得金額" value={rt.taxableOrdinary} />
            {rt.taxableSeparate > 0 && <Row label="課税分離所得（株式等）" value={rt.taxableSeparate} />}
            <Row label="所得割（税額控除前）" value={rt.incomeLevyBeforeCredits} />
            {rt.credits.length > 0 && <tr className="group"><td colSpan={2}>税額控除</td></tr>}
            <Rows items={rt.credits} />
            <Row label={`所得割${rt.incomeLevyExempt ? "（非課税）" : ""}`} value={rt.incomeLevy} />
            <Row label={`均等割（森林環境税含む）${rt.perCapitaExempt ? "（非課税）" : ""}`} value={rt.perCapita} />
            <Row label="住民税 合計" value={rt.total} strong />
          </tbody>
        </table>
        {rt.total > 0 && (
          <p className="meta">
            給与から天引き（特別徴収）の場合：2027年6月分 {yen(rt.monthlyFirst)}、7月〜翌年5月分 各 {yen(rt.monthlyRest)}
          </p>
        )}
      </details>

      {(result.stockWithholding.incomeTax > 0 || result.stockWithholding.residentTax > 0) && (
        <details>
          <summary>申告不要とした株式の源泉徴収税</summary>
          <table>
            <tbody>
              <Row label="所得税・復興税（15.315%）" value={result.stockWithholding.incomeTax} />
              <Row label="住民税（5%）" value={result.stockWithholding.residentTax} />
            </tbody>
          </table>
        </details>
      )}

      {result.savings.length > 0 && (
        <details open>
          <summary>控除による節税額</summary>
          <table className="compare">
            <thead>
              <tr>
                <th>控除</th>
                <th className="num">所得税</th>
                <th className="num">住民税</th>
                <th className="num">合計</th>
              </tr>
            </thead>
            <tbody>
              {result.savings.map((sv) => (
                <tr key={sv.label}>
                  <td>{sv.label}</td>
                  <td className="num">{yen(sv.incomeTax)}</td>
                  <td className="num">{yen(sv.residentTax)}</td>
                  <td className="num">{yen(sv.total)}</td>
                </tr>
              ))}
            </tbody>
          </table>
          <p className="meta">
            その控除だけがなかった場合と比べて減っている税額です。1つずつ比べているので、行を足した額は全部をやめた場合の差とは一致しないことがあります。ふるさと納税は上の欄を見てください。
          </p>
        </details>
      )}

      {result.stockOptions.length > 0 && <StockOptions options={result.stockOptions} />}

      <p className="disclaimer">
        本アプリの結果は概算です。実際の税額は勤務先の年末調整・確定申告・自治体の賦課決定によります。
      </p>
    </div>
  );
}

import { useEffect, useState } from "react";
import { Area, AreaChart, CartesianGrid, ReferenceLine, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";
import { CurvePoint } from "../types";
import { yen } from "./Fields";

/** 積み上げる系列。色は App.css の --series-* （ライト／ダークで別の値）を使う */
const SERIES = [
  { key: "takeHome", label: "手取り", cssVar: "--series-1" },
  { key: "socialInsurance", label: "社会保険料", cssVar: "--series-2" },
  { key: "incomeTax", label: "所得税", cssVar: "--series-3" },
  { key: "residentTax", label: "住民税", cssVar: "--series-4" },
] as const;

const COLOR_VARS = [...SERIES.map((s) => s.cssVar), "--surface", "--border", "--muted", "--text"] as const;
type Colors = Record<(typeof COLOR_VARS)[number], string>;

const readColors = (): Colors => {
  const style = getComputedStyle(document.documentElement);
  return Object.fromEntries(COLOR_VARS.map((v) => [v, style.getPropertyValue(v).trim()])) as Colors;
};

/** CSS 変数の色を読む。OS のライト／ダークが切り替わったら読み直す */
function useColors(): Colors {
  const [colors, setColors] = useState(readColors);
  useEffect(() => {
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const update = () => setColors(readColors());
    mq.addEventListener("change", update);
    return () => mq.removeEventListener("change", update);
  }, []);
  return colors;
}

/** 万円単位の表記（小数1桁まで。例：1,134,567 → 113.5万） */
const man = (n: number) => `${(n / 10_000).toLocaleString("ja-JP", { maximumFractionDigits: 1 })}万`;

type TooltipProps = { active?: boolean; payload?: { payload: CurvePoint }[] };

function ChartTooltip({ active, payload }: TooltipProps) {
  const p = active ? payload?.[0]?.payload : undefined;
  if (!p) return null;
  return (
    <div className="chart-tooltip">
      <div className="chart-tooltip-title">給与収入 {yen(p.salary)}</div>
      {SERIES.map((s) => (
        <div className="chart-tooltip-row" key={s.key}>
          <span className={`swatch ${s.key}`} />
          <span>{s.label}</span>
          <span className="num">{yen(p[s.key])}</span>
        </div>
      ))}
    </div>
  );
}

/** 給与収入を変えたときの、手取り・社会保険料・所得税・住民税の積み上げグラフ */
export function IncomeChart({ curve, currentSalary }: { curve: CurvePoint[]; currentSalary: number }) {
  const colors = useColors();
  if (curve.length === 0) return null;
  const tableRows = curve.filter((p) => p.salary > 0 && p.salary % 1_000_000 === 0);

  return (
    <details open>
      <summary>年収別の手取り</summary>
      <ul className="chart-legend">
        {SERIES.map((s) => (
          <li key={s.key}>
            <span className={`swatch ${s.key}`} />
            {s.label}
          </li>
        ))}
      </ul>
      <div className="chart">
        <ResponsiveContainer width="100%" height="100%">
          <AreaChart data={curve} margin={{ top: 8, right: 8, bottom: 0, left: 0 }}>
            <CartesianGrid stroke={colors["--border"]} vertical={false} />
            <XAxis
              dataKey="salary"
              type="number"
              domain={[0, "dataMax"]}
              tickFormatter={man}
              tick={{ fill: colors["--muted"], fontSize: 11 }}
              stroke={colors["--border"]}
              tickCount={6}
            />
            <YAxis
              domain={[0, "auto"]}
              tickFormatter={man}
              tick={{ fill: colors["--muted"], fontSize: 11 }}
              stroke={colors["--border"]}
              width={52}
            />
            <Tooltip content={<ChartTooltip />} cursor={{ stroke: colors["--muted"], strokeWidth: 1 }} />
            {SERIES.map((s) => (
              <Area
                key={s.key}
                dataKey={s.key}
                name={s.label}
                stackId="total"
                type="linear"
                fill={colors[s.cssVar]}
                fillOpacity={1}
                // 面どうしの境目は、背景色の細い線で区切る
                stroke={colors["--surface"]}
                strokeWidth={2}
                isAnimationActive={false}
              />
            ))}
            {currentSalary > 0 && (
              <ReferenceLine
                x={currentSalary}
                stroke={colors["--text"]}
                strokeDasharray="4 3"
                label={{ value: "現在", position: "insideTopRight", fill: colors["--text"], fontSize: 11 }}
              />
            )}
          </AreaChart>
        </ResponsiveContainer>
      </div>
      <p className="meta">
        給与収入だけを変え、ほかの入力はそのままにした場合です。社会保険料は、月額給与と賞与（手入力の場合は金額）を給与収入に比例させて概算しています。申告不要とした株式の源泉徴収税は、所得税・住民税に含めています。
      </p>
      <details>
        <summary>表で見る</summary>
        <table className="compare">
          <thead>
            <tr>
              <th>給与収入</th>
              {SERIES.map((s) => (
                <th className="num" key={s.key}>
                  {s.label}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {tableRows.map((p) => (
              <tr key={p.salary} className={p.salary === currentSalary ? "current" : undefined}>
                <td>{man(p.salary)}</td>
                {SERIES.map((s) => (
                  <td className="num" key={s.key}>
                    {man(p[s.key])}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </details>
    </details>
  );
}

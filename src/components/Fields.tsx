import { ReactNode, useEffect, useState } from "react";

export const yen = (n: number) => n.toLocaleString("ja-JP") + " 円";

const fmt = (n: number) => (n === 0 ? "" : n.toLocaleString("ja-JP"));

/** 3桁区切り表示の金額入力 */
export function MoneyInput(props: {
  label: string;
  value: number;
  onChange: (v: number) => void;
  hint?: ReactNode;
  allowNegative?: boolean;
  suffix?: string;
}) {
  const { label, value, onChange, hint, allowNegative, suffix = "円" } = props;
  const [text, setText] = useState(fmt(value));
  const [focused, setFocused] = useState(false);

  useEffect(() => {
    if (!focused) setText(fmt(value));
  }, [value, focused]);

  const handle = (raw: string) => {
    const cleaned = raw.replace(/[,，\s]/g, "").replace(/[０-９]/g, (c) =>
      String.fromCharCode(c.charCodeAt(0) - 0xfee0),
    );
    const pattern = allowNegative ? /^-?\d*$/ : /^\d*$/;
    if (!pattern.test(cleaned)) return;
    setText(cleaned);
    const n = Number(cleaned);
    onChange(Number.isFinite(n) ? n : 0);
  };

  return (
    <label className="field">
      <span className="field-label">{label}</span>
      <span className="field-input">
        <input
          type="text"
          inputMode="numeric"
          value={text}
          placeholder="0"
          onFocus={() => {
            setFocused(true);
            setText(value === 0 ? "" : String(value));
          }}
          onBlur={() => setFocused(false)}
          onChange={(e) => handle(e.target.value)}
        />
        <span className="suffix">{suffix}</span>
      </span>
      {hint && <span className="field-hint">{hint}</span>}
    </label>
  );
}

export function NumberInput(props: {
  label: string;
  value: number;
  onChange: (v: number) => void;
  min?: number;
  max?: number;
  suffix?: string;
  hint?: ReactNode;
}) {
  const { label, value, onChange, min = 0, max = 120, suffix, hint } = props;
  return (
    <label className="field">
      <span className="field-label">{label}</span>
      <span className="field-input">
        <input
          type="number"
          value={value}
          min={min}
          max={max}
          onChange={(e) => {
            const n = Math.trunc(Number(e.target.value));
            onChange(Number.isFinite(n) ? Math.min(max, Math.max(min, n)) : min);
          }}
        />
        {suffix && <span className="suffix">{suffix}</span>}
      </span>
      {hint && <span className="field-hint">{hint}</span>}
    </label>
  );
}

export function Select<T extends string>(props: {
  label: string;
  value: T;
  options: { value: T; label: string }[];
  onChange: (v: T) => void;
  hint?: ReactNode;
}) {
  return (
    <label className="field">
      <span className="field-label">{props.label}</span>
      <select value={props.value} onChange={(e) => props.onChange(e.target.value as T)}>
        {props.options.map((o) => (
          <option key={o.value} value={o.value}>
            {o.label}
          </option>
        ))}
      </select>
      {props.hint && <span className="field-hint">{props.hint}</span>}
    </label>
  );
}

export function Check(props: {
  label: string;
  checked: boolean;
  onChange: (v: boolean) => void;
  hint?: ReactNode;
}) {
  return (
    <label className="check">
      <input
        type="checkbox"
        checked={props.checked}
        onChange={(e) => props.onChange(e.target.checked)}
      />
      <span>
        {props.label}
        {props.hint && <span className="field-hint">{props.hint}</span>}
      </span>
    </label>
  );
}

export function Section(props: {
  title: string;
  badge?: string;
  defaultOpen?: boolean;
  children: ReactNode;
}) {
  return (
    <details className="section" open={props.defaultOpen}>
      <summary>
        <span>{props.title}</span>
        {props.badge && <span className="badge">{props.badge}</span>}
      </summary>
      <div className="section-body">{props.children}</div>
    </details>
  );
}

export const DISABILITY_OPTIONS = [
  { value: "none" as const, label: "なし" },
  { value: "general" as const, label: "障害者" },
  { value: "special" as const, label: "特別障害者" },
  { value: "specialCohabiting" as const, label: "同居特別障害者" },
];

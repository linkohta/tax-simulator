import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { InputForm } from "./components/InputForm";
import { ResultPanel } from "./components/ResultPanel";
import { yen } from "./components/Fields";
import { CurvePoint, TaxInput, TaxResult, defaultInput } from "./types";
import "./App.css";

/** 狭い画面（スマホ）で表示するペイン。広い画面では両方を並べるので使わない */
type MobileView = "input" | "result";

/** 入力が止まってから下書きを自動保存するまでの時間 */
const DRAFT_SAVE_DELAY_MS = 800;

function App() {
  const [input, setInputState] = useState<TaxInput>(defaultInput);
  const [result, setResult] = useState<TaxResult | null>(null);
  const [curve, setCurve] = useState<CurvePoint[]>([]);
  const [prefectures, setPrefectures] = useState<string[]>([]);
  const [message, setMessage] = useState<string | null>(null);
  const [calcError, setCalcError] = useState<string | null>(null);
  const [view, setView] = useState<MobileView>("input");
  const seq = useRef(0);

  const setInput = (updater: (prev: TaxInput) => TaxInput) => setInputState(updater);

  useEffect(() => {
    invoke<string[]>("prefectures").then(setPrefectures).catch(() => setPrefectures(["東京都"]));
  }, []);

  // 入力変更のたびに再計算（軽いデバウンス）
  useEffect(() => {
    const id = ++seq.current;
    const t = setTimeout(() => {
      invoke<TaxResult>("calculate", { input })
        .then((r) => {
          if (id === seq.current) {
            setResult(r);
            setCalcError(null);
          }
        })
        .catch((e) => setCalcError(`計算エラー: ${e}`));
      // 年収別のグラフ用。失敗してもグラフが出ないだけなので、エラーにはしない
      invoke<CurvePoint[]>("salary_curve", { input })
        .then((c) => {
          if (id === seq.current) setCurve(c);
        })
        .catch(() => setCurve([]));
    }, 120);
    return () => clearTimeout(t);
  }, [input]);

  // 起動時に前回の入力（下書き）を復元する。復元が済むまでは自動保存しない（初期値で上書きしないため）
  const [restored, setRestored] = useState(false);
  useEffect(() => {
    invoke<TaxInput | null>("load_draft")
      .then((draft) => {
        if (draft) setInputState(draft);
      })
      .catch((e) => flash(`前回の入力を復元できませんでした: ${e}`))
      .finally(() => setRestored(true));
  }, []);

  // 入力が変わったら少し待って下書きを保存する。保存先は Rust 側で決める（Webview からパスを渡さない）
  const latestInput = useRef(input);
  latestInput.current = input;
  const draftErrorShown = useRef(false);
  const saveDraft = () =>
    invoke("save_draft", { input: latestInput.current }).catch((e) => {
      // 失敗のたびに出すと邪魔なので、最初の1回だけ知らせる
      if (!draftErrorShown.current) flash(`入力の自動保存に失敗しました: ${e}`);
      draftErrorShown.current = true;
    });
  useEffect(() => {
    if (!restored) return;
    const t = setTimeout(saveDraft, DRAFT_SAVE_DELAY_MS);
    return () => clearTimeout(t);
  }, [input, restored]);
  // アプリが裏に回るとき（スマホでの切り替えなど）は待たずに保存する
  useEffect(() => {
    if (!restored) return;
    const onHide = () => {
      if (document.visibilityState === "hidden") saveDraft();
    };
    document.addEventListener("visibilitychange", onHide);
    return () => document.removeEventListener("visibilitychange", onHide);
  }, [restored]);

  // リセットは2回押して実行する（1回目で確認表示に変わり、数秒で元に戻る）
  const [confirmingReset, setConfirmingReset] = useState(false);
  useEffect(() => {
    if (!confirmingReset) return;
    const t = setTimeout(() => setConfirmingReset(false), 4000);
    return () => clearTimeout(t);
  }, [confirmingReset]);
  const onReset = () => {
    if (!confirmingReset) {
      setConfirmingReset(true);
      return;
    }
    setConfirmingReset(false);
    setInputState(defaultInput());
    flash("入力をリセットしました");
  };

  const flash = (m: string) => {
    setMessage(m);
    setTimeout(() => setMessage(null), 3000);
  };

  // 保存先・読込元のダイアログは Rust 側で開く（Webview からパスを渡さない）
  const onSave = async () => {
    try {
      const saved = await invoke<boolean>("save_scenario", { input });
      if (saved) flash("保存しました");
    } catch (e) {
      flash(String(e));
    }
  };

  const onLoad = async () => {
    try {
      const loaded = await invoke<TaxInput | null>("load_scenario");
      if (!loaded) return;
      setInputState(loaded);
      flash("読み込みました");
    } catch (e) {
      flash(String(e));
    }
  };

  return (
    <div className="app">
      <header>
        <div>
          <h1>所得税・住民税シミュレーター</h1>
          <p>令和8年（2026年）分所得税 ／ 令和9年度住民税（2026年10月時点の税制）</p>
        </div>
        <div className="actions">
          {calcError && <span className="toast error">{calcError}</span>}
          {message && <span className="toast">{message}</span>}
          <button type="button" onClick={onLoad}>読込</button>
          <button type="button" onClick={onSave}>保存</button>
          <button type="button" className={confirmingReset ? "ghost confirm" : "ghost"} onClick={onReset}>
            {confirmingReset ? "もう一度押すと消去" : "リセット"}
          </button>
        </div>
      </header>
      <main data-view={view}>
        <InputForm input={input} setInput={setInput} prefectures={prefectures} />
        <ResultPanel result={result} curve={curve} currentSalary={input.salary.annualIncome} />
      </main>
      <nav className="tabbar" aria-label="表示の切り替え">
        <button
          type="button"
          className={view === "input" ? "active" : undefined}
          aria-pressed={view === "input"}
          onClick={() => setView("input")}
        >
          入力
        </button>
        <button
          type="button"
          className={view === "result" ? "active" : undefined}
          aria-pressed={view === "result"}
          onClick={() => setView("result")}
        >
          結果
          {result && <span className="tab-sub">手取り {yen(result.summary.takeHome)}</span>}
        </button>
      </nav>
    </div>
  );
}

export default App;

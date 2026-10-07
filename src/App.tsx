import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { InputForm } from "./components/InputForm";
import { ResultPanel } from "./components/ResultPanel";
import { yen } from "./components/Fields";
import { TaxInput, TaxResult, defaultInput } from "./types";
import "./App.css";

const FILTERS = [{ name: "税シミュレーション", extensions: ["json"] }];

/** 狭い画面（スマホ）で表示するペイン。広い画面では両方を並べるので使わない */
type MobileView = "input" | "result";

function App() {
  const [input, setInputState] = useState<TaxInput>(defaultInput);
  const [result, setResult] = useState<TaxResult | null>(null);
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
    }, 120);
    return () => clearTimeout(t);
  }, [input]);

  const flash = (m: string) => {
    setMessage(m);
    setTimeout(() => setMessage(null), 3000);
  };

  const onSave = async () => {
    const path = await save({ filters: FILTERS, defaultPath: "tax-scenario.json" });
    if (!path) return;
    try {
      await invoke("save_scenario", { path, input });
      flash("保存しました");
    } catch (e) {
      flash(String(e));
    }
  };

  const onLoad = async () => {
    const path = await open({ filters: FILTERS, multiple: false, directory: false });
    if (!path) return;
    try {
      const loaded = await invoke<TaxInput>("load_scenario", { path });
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
          <button type="button" className="ghost" onClick={() => setInputState(defaultInput())}>
            リセット
          </button>
        </div>
      </header>
      <main data-view={view}>
        <InputForm input={input} setInput={setInput} prefectures={prefectures} />
        <ResultPanel result={result} />
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

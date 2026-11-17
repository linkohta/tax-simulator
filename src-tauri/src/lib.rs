pub mod tax;

use std::io::Write;
use std::path::PathBuf;

use tauri::{AppHandle, Manager, Runtime, Window};
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder};
use tauri_plugin_fs::{FsExt, OpenOptions};
use tax::model::{TaxInput, TaxResult};

/// 保存・読込ダイアログで選べるファイルの種類
const SCENARIO_FILTER_NAME: &str = "税シミュレーション";
const SCENARIO_EXTENSIONS: &[&str] = &["json"];
/// 保存ダイアログに最初から入れておくファイル名
const SCENARIO_DEFAULT_FILE_NAME: &str = "tax-scenario.json";
/// 自動保存する下書きのファイル名（アプリのデータフォルダに置く）
const DRAFT_FILE_NAME: &str = "draft.json";

/// 税額を計算する
#[tauri::command]
fn calculate(input: TaxInput) -> TaxResult {
    tax::calculate(&input)
}

/// 協会けんぽの都道府県一覧（概算用）
#[tauri::command]
fn prefectures() -> Vec<&'static str> {
    tax::params::KYOKAI_KENPO_RATES.iter().map(|(p, _)| *p).collect()
}

/// シナリオファイル用のダイアログ（JSON に絞り込む）
fn scenario_dialog<R: Runtime>(window: &Window<R>) -> FileDialogBuilder<R> {
    window.dialog().file().add_filter(SCENARIO_FILTER_NAME, SCENARIO_EXTENSIONS)
}

/// 保存ダイアログを開き、選ばれたファイルにシナリオ（入力内容）を JSON で保存する。キャンセルしたら `false`
///
/// 保存先は Webview から受け取らず、このコマンドで開くダイアログでユーザーが選んだものに限る。
/// 画面側に不正なスクリプトが紛れ込んでも、任意のファイルを書き換えられないようにするため。
/// Android ではファイル選択画面が返す `content://` URI に書き込む。
#[tauri::command]
async fn save_scenario(window: Window, input: TaxInput) -> Result<bool, String> {
    let json = serde_json::to_string_pretty(&input).map_err(|e| e.to_string())?;
    let dialog = scenario_dialog(&window).set_file_name(SCENARIO_DEFAULT_FILE_NAME);
    #[cfg(desktop)]
    let dialog = dialog.set_parent(&window);
    // async コマンドは非同期ランタイムのスレッドで動くので、メインスレッドを止めずにダイアログを待てる
    let Some(path) = dialog.blocking_save_file() else {
        return Ok(false);
    };
    let mut opts = OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    window
        .fs()
        .open(path, opts)
        .and_then(|mut f| f.write_all(json.as_bytes()))
        .map_err(|e| format!("保存に失敗しました: {e}"))?;
    Ok(true)
}

/// ファイル選択ダイアログを開き、選ばれた JSON ファイルからシナリオを読み込む。キャンセルしたら `None`
///
/// `save_scenario` と同じく、読み込むのはユーザーがダイアログで選んだファイル（`content://` URI を含む）に限る。
#[tauri::command]
async fn load_scenario(window: Window) -> Result<Option<TaxInput>, String> {
    let dialog = scenario_dialog(&window);
    // tauri-plugin-dialog の open コマンドに合わせ、Linux では親ウィンドウを指定しない
    #[cfg(any(windows, target_os = "macos"))]
    let dialog = dialog.set_parent(&window);
    let Some(path) = dialog.blocking_pick_file() else {
        return Ok(None);
    };
    let text = window
        .fs()
        .read_to_string(path)
        .map_err(|e| format!("読み込みに失敗しました: {e}"))?;
    // 旧形式の項目（合算の譲渡損益など）は現在の形式に直して画面に渡す
    serde_json::from_str::<TaxInput>(&text)
        .map(|input| Some(input.normalized()))
        .map_err(|e| format!("ファイル形式が正しくありません: {e}"))
}

/// 下書きのファイルの場所（アプリのデータフォルダ内で固定。Webview からは指定できない）
fn draft_path<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join(DRAFT_FILE_NAME))
        .map_err(|e| format!("保存先を取得できません: {e}"))
}

/// 入力内容を下書きとして自動保存する（次の起動時に `load_draft` で復元する）
#[tauri::command]
async fn save_draft(app: AppHandle, input: TaxInput) -> Result<(), String> {
    let path = draft_path(&app)?;
    let json = serde_json::to_string(&input).map_err(|e| e.to_string())?;
    // フォルダの作成は tauri_plugin_fs に API がないので std::fs を使う（アプリ専用のフォルダで、content:// URI にはならない）
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("保存先を作成できません: {e}"))?;
    }
    let mut opts = OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    app.fs()
        .open(path, opts)
        .and_then(|mut f| f.write_all(json.as_bytes()))
        .map_err(|e| format!("自動保存に失敗しました: {e}"))
}

/// 自動保存した下書きを読み込む。まだなければ `None`
#[tauri::command]
async fn load_draft(app: AppHandle) -> Result<Option<TaxInput>, String> {
    let path = draft_path(&app)?;
    if !path.exists() {
        return Ok(None);
    }
    let text = app.fs().read_to_string(path).map_err(|e| format!("読み込みに失敗しました: {e}"))?;
    serde_json::from_str::<TaxInput>(&text)
        .map(|input| Some(input.normalized()))
        .map_err(|e| format!("下書きの形式が正しくありません: {e}"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            calculate,
            prefectures,
            save_scenario,
            load_scenario,
            save_draft,
            load_draft
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

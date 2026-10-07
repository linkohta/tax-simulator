pub mod tax;

use std::io::Write;

use tauri::AppHandle;
use tauri_plugin_fs::{FilePath, FsExt, OpenOptions};
use tax::model::{TaxInput, TaxResult};

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

/// シナリオ（入力内容）を JSON ファイルに保存する
///
/// `path` は通常のファイルパスのほか、Android のファイルピッカーが返す `content://` URI も受け付ける。
#[tauri::command]
fn save_scenario(app: AppHandle, path: FilePath, input: TaxInput) -> Result<(), String> {
    let json = serde_json::to_string_pretty(&input).map_err(|e| e.to_string())?;
    let mut opts = OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    app.fs()
        .open(path, opts)
        .and_then(|mut f| f.write_all(json.as_bytes()))
        .map_err(|e| format!("保存に失敗しました: {e}"))
}

/// JSON ファイルからシナリオを読み込む（`content://` URI にも対応）
#[tauri::command]
fn load_scenario(app: AppHandle, path: FilePath) -> Result<TaxInput, String> {
    let text = app
        .fs()
        .read_to_string(path)
        .map_err(|e| format!("読み込みに失敗しました: {e}"))?;
    serde_json::from_str(&text).map_err(|e| format!("ファイル形式が正しくありません: {e}"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            calculate,
            prefectures,
            save_scenario,
            load_scenario
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

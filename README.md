# 所得税・住民税シミュレーター

令和8年（2026年）分の所得税と、その所得にかかる令和9年度の住民税を試算するアプリです（Rust + Tauri 2 + React/TypeScript）。Windows などのデスクトップと Android で動きます。2026年10月時点の税制で計算します。

## 起動

```bash
npm install
npm run tauri dev
```

計算エンジンのテスト：

```bash
cd src-tauri && cargo test --lib
```

## Android 版

パッケージ名は `io.github.masterlink.taxsimulator`。Android プロジェクトは `src-tauri/gen/android/` にあります（`tauri android init` で生成し、署名・余白処理などを手で加えているので、再生成しないこと）。

### 必要なもの

- Android Studio（SDK と NDK）、JDK 17 以上
- 環境変数 `ANDROID_HOME`（例：`%LOCALAPPDATA%\Android\Sdk`）、`NDK_HOME`（例：`%ANDROID_HOME%\ndk\27.0.12077973`）、`JAVA_HOME`
- Rust のターゲット：

  ```bash
  rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
  ```

### 実行とビルド

```bash
npm run tauri android dev          # 実機（USB デバッグ）またはエミュレータで起動
npm run tauri android build -- --aab   # Google Play 用の AAB（リリース版）
npm run tauri android build -- --apk   # 配布確認用の APK
```

AAB は `src-tauri/gen/android/app/build/outputs/bundle/universalRelease/` に出力されます。

### アップロード鍵（署名）

鍵とパスワードは git に入れません（`.gitignore` 済み）。初回だけ次の手順で作ります。鍵をなくすと Google Play で更新できなくなるので、安全な場所にバックアップしてください。

1. 鍵を作る（パスワードを聞かれます）：

   ```bash
   keytool -genkey -v -keystore %USERPROFILE%\upload-keystore.jks -keyalg RSA -keysize 2048 -validity 10000 -alias upload
   ```

2. `src-tauri/gen/android/keystore.properties` を作る：

   ```properties
   storeFile=C:\\Users\\<ユーザー名>\\upload-keystore.jks
   storePassword=<鍵ストアのパスワード>
   keyAlias=upload
   keyPassword=<鍵のパスワード>
   ```

このファイルがあるとリリース版に署名し、ないときは署名なしでビルドします。

### バージョン

`src-tauri/tauri.conf.json` の `version` から versionName と versionCode が決まります（例：`0.1.0` → versionCode 1000）。Google Play に上げ直すたびに `version` を上げてください。

### スマホでの表示

画面幅 860px 以下では、入力と結果を下部のタブで切り替えます。保存・読込は Android のファイル選択画面を使います。

## 構成

| パス | 内容 |
|---|---|
| `src-tauri/src/tax/params.rs` | 税率・控除額・社会保険料率などのパラメータ（制度改正時はここを修正） |
| `src-tauri/src/tax/mod.rs` | 所得税・住民税の計算、ふるさと納税上限の探索 |
| `src-tauri/src/tax/deductions.rs` | 給与所得控除、人的控除、保険料控除、医療費控除 |
| `src-tauri/src/tax/social.rs` | 社会保険料の概算（協会けんぽ・厚生年金・雇用保険） |
| `src-tauri/src/tax/tests.rs` | 手計算した例と照合するテスト |
| `src/` | 画面（React） |

## 反映している主な制度

- 令和8年度改正の基礎控除：62万円に、令和8・9年分の特例加算（合計所得489万円以下は104万円、655万円以下は67万円）を上乗せ
- 給与所得控除の最低保障74万円と、措法29条の4の端数表（所得税・住民税の両方）
- 配偶者・扶養親族の所得要件62万円、特定親族特別控除、勤労学生の所得要件89万円
- 一般生命保険料控除の上限を6万円にする特例（23歳未満の扶養親族がいる場合。所得税のみ）
- ふるさと納税の人的控除差調整額に「所得税の基礎控除 − 48万円」を加算
- 住宅ローン控除の住民税控除限度額（居住開始年による違い）
- 協会けんぽの令和8年度都道府県別保険料率、介護保険料率1.62%、子ども・子育て支援金0.23%、雇用保険料率（労働者負担）5/1000

## 簡略化していること

- 住民税は標準税率（市6%・県4%）。指定都市（市8%・県2%）や超過課税は考慮していません。
- 住民税の非課税限度額は1級地の基準で判定します。
- 社会保険料の概算は、年間を通して同じ標準報酬月額と令和8年度の料率で計算します。
- 配当控除は「剰余金の配当等」の率で計算します。投資信託の分配金の率は扱いません。
- 「極めて高い水準の所得に対する負担の適正化措置」（ミニマム税）は計算に含めず、該当しそうな場合に警告を出します。
- 住宅ローン控除は、計算済みの控除可能額を入力する方式です。

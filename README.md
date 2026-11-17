# 所得税・住民税シミュレーター

令和8年（2026年）分の所得税と、その所得にかかる令和9年度の住民税を試算するアプリです（Rust + Tauri 2 + React/TypeScript）。Windows などのデスクトップと Android で動きます。2026年10月時点の税制で計算します。

## 起動

```bash
npm install
npm run app:dev
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
| `src-tauri/src/tax/stocks.rs` | 株式の配当・譲渡損益（口座ごとの損益、損益通算、繰越控除、配当控除） |
| `src-tauri/src/tax/comparison.rs` | 株式の課税方式ごとの税額の比較 |
| `src-tauri/src/tax/foreign_tax.rs` | 外国税額控除 |
| `src-tauri/src/tax/housing_loan.rs` | 住宅ローン控除の住民税からの控除限度額 |
| `src-tauri/src/tax/validation.rs` | 入力の矛盾などの警告 |
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
- 配当控除は、剰余金の配当等（10%）・証券投資信託（5%）・外貨建等証券投資信託（2.5%）の区分ごとに入力した額で計算します。課税総所得金額等が1,000万円を超える場合、証券投資信託の2区分の間では外貨建等を上に積んで計算します。
- 株式の譲渡損益は口座ごとに入力します。源泉徴収ありの特定口座は申告するかどうかを選べ、源泉徴収なしの特定口座と一般口座は損益があれば申告するものとします（給与以外の所得が20万円以下の場合の申告不要は扱いません）。
- 源泉徴収ありの特定口座に配当を受け入れている場合の、口座内での配当と譲渡損失の損益通算は計算しません（申告分離で申告した場合の税額は「株式の課税方式の比較」で確認できます）。
- 外国税額控除は、その年の外国所得税額と国外所得金額から控除限度額を計算します。控除しきれない額の繰越（3年）と、指定都市の住民税の率（6%・24%）は扱いません。投資信託の二重課税調整は扱いません。
- 「株式の課税方式の比較」は税額だけを比べます。申告による国民健康保険料や扶養の判定への影響は含みません。
- 「極めて高い水準の所得に対する負担の適正化措置」（ミニマム税）は計算に含めず、該当しそうな場合に警告を出します。
- 社会保険料の概算は、年齢を健康保険（75歳未満）・厚生年金（70歳未満）・介護保険（40〜64歳）の判定に使います。65歳以上の介護保険料と後期高齢者医療の保険料は含めません（「その他に支払った社会保険料」に入力）。
- 住宅ローン控除は、計算済みの控除可能額を入力する方式です。住民税から控除できる限度額は、入居時期に応じて5%（最高97,500円）か7%（最高136,500円）を選びます。

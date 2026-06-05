# 外部設計

プロジェクト: 労務コンパス / LaborLens

## 1. 目的

この文書は、画面、レポート、表示順、操作フロー、利用者への見せ方を定義する。内部 DB、計算ロジック、抑制判定の詳細は `DATA-DESIGN.md`、`BUSINESS-RULES.md`、`ARCHITECTURE.md` に分離する。

## 2. UI 実装スタック

将来の local UI は、次の構成で実装する。

| 領域 | 採用技術 | 用途 |
| --- | --- | --- |
| desktop shell | Tauri | installer 付き desktop app、起動導線、OS file dialog、local server / worker 起動管理 |
| frontend build | Vite + React + TypeScript | local UI の画面実装、型付き UI state、開発時 hot reload |
| UI components | shadcn/ui | 業務アプリ向けの button、table、tabs、dialog、sidebar、form、toast などの基本部品 |
| data table | TanStack Table | issues、readiness、artifact、run history、修正依頼一覧などの表表示、sort、filter、pagination |
| server state | TanStack Query | Local Server API から取得する run、progress、report、artifact、use case、guide response の cache と再取得制御 |
| AI assistant UI | assistant-ui | ガイド AI の thread、message、composer、tool/result 表示 |
| AI connection | local server 経由の Ollama | UI から Ollama を直接呼ばず、Local Server API が許可済み文脈だけを渡す |

### 2.1 採用理由

LaborLens の UI は、チャット中心のアプリではなく、CSV 取込、進捗確認、issue 表、レポート閲覧、再確認結果を扱う業務アプリである。そのため、画面全体は Tauri + React + shadcn/ui + TanStack 系で構成し、AI assistant は補助 UI として assistant-ui を組み込む。

TanStack Table は、`issues.csv` 相当の行、列、理由、優先度、readiness effect、RunId などを安定して表示するために使う。TanStack Query は、ジョブ進捗や成果物参照のように Local Server API から継続的に取得する状態を UI state と分離するために使う。

assistant-ui は、操作説明、issue の意味、ready / partial / blocked の説明、抑制理由の説明を扱う。AI assistant は判断代替を行わず、Local Server API が渡す承認済み文書、版管理済み文書、抑制後レポート、RuleExplanation だけを根拠にする。

### 2.2 Desktop 配布方針

最終的な local UI は、Tauri + React により Windows desktop app として installer 付きで配布する。利用者は browser で `localhost` を直接開くのではなく、インストール済みアプリを起動して利用する。

配布単位は、次をまとめることを目標にする。

- Tauri desktop shell
- React build 済み UI asset
- Local Server API executable または同等の Tauri sidecar / Rust command boundary
- Background Worker executable または同等の worker boundary
- PostgreSQL runtime / managed local cluster helper
- DB migration / seed / initialization helper
- Source Archive / Artifact Store の初期 directory 作成
- Ollama runtime
- 初期モデル `qwen3:8b`
- Ollama 接続確認、model availability 確認、model 更新確認の UI

Tauri は UI が PostgreSQL、Source Archive、Artifact Store、Ollama に直接接続するためには使わない。Tauri app から見ても、安全境界は Local Server API が担当する。Tauri は配布、起動、file dialog、アプリ設定、通知、ローカルプロセス起動管理を担当する。

初期開発では Vite dev server と Rust local server を個別に起動してよい。ただし、本番配布では Tauri installer から起動できる単体 desktop app を目標にする。

PostgreSQL と Ollama は、利用者が別途手動インストールしなくても初回起動できるよう installer 同梱を目標にする。PostgreSQL data directory、Ollama model directory、Source Archive、Artifact Store は、アプリ管理下の local data directory に分離して配置する。

PostgreSQL と Ollama を同梱しても、UI がそれらへ直接接続してよいわけではない。UI と Tauri shell は Local Server API を経由し、Local Server API / worker が DB 接続、Ollama 接続、許可済み文脈、ログマスキングを管理する。

### 2.3 UI から直接接続しないもの

UI は次へ直接接続してはならない。

- PostgreSQL
- Source Archive
- Artifact Store の実ファイル
- 抑制前データ
- Ollama
- RAG index

UI は必ず Local Server API を経由する。Ollama assistant についても、ブラウザから `http://127.0.0.1:11434` へ直接接続せず、Local Server API が安全境界、許可済み文脈、ログマスキング、プロンプト注入対策を担う。

## 3. 画面分割方針

LaborLens の UI は、CSV 取込、ジョブ進捗、データ準備状況、不備一覧、レポート閲覧、再確認、プライバシー抑制、ガイド AI を扱う。これらは利用目的と情報密度が異なるため、単一画面へ詰め込まない。

初期画面は全情報の一覧ではなく、利用者が次に確認すべき場所へ進める実行サマリーとする。詳細情報はページ、タブ、ドロワー、必要に応じた別タブ表示へ分ける。

### 3.1 基本ナビゲーション

local UI は、左サイドバーまたは同等の永続ナビゲーションで主要領域を分ける。

| 領域 | 主な内容 | 表示方針 |
| --- | --- | --- |
| 実行 | CSV 選択、実行設定、run 開始、進捗 | 操作と状態確認を中心にする |
| 結果 | データ準備状況、不備件数、成果物一覧、次の確認 | run 完了後に最初に見るサマリーにする |
| 不備一覧 | issue table、修正対象、優先度、行・列・理由 | TanStack Table で高密度な作業表にする |
| レポート | 月次労務、勤怠確認、人件費確認、抑制済み集計 | 読む画面として余白、見出し、タブ切替を優先する |
| 再確認 | 修正前後の RunId、入力ハッシュ、issue 件数比較 | 比較専用画面にし、原本保護を確認できるようにする |
| ガイド | AI assistant、RuleExplanation、根拠文書 | 補助説明として扱い、判断代替をさせない |
| 設定 | CSV 読み込み設定、表示順、ローカル接続設定 | 通常業務画面から分離する |

### 3.2 ページ、タブ、ドロワー、別タブの使い分け

| 表示方法 | 使う場面 |
| --- | --- |
| ページ遷移 | CSV 取込、不備一覧、レポート、再確認など、作業目的が変わる場合 |
| タブ | 同じ成果物や同じ run の中で、月次、勤怠、人件費、抑制済み集計などを切り替える場合 |
| ドロワー | issue の詳細、根拠行、抑制理由、入力ハッシュなど、一覧から一時的に掘り下げる場合 |
| モーダル | run 開始確認、再実行確認、取り消し確認など、短い確認操作だけに使う場合 |
| 別タブまたは別ウィンドウ | PDF、Markdown レポート、長い比較レポートなど、読み物として独立して確認する場合 |

### 3.3 一画面に混在させない情報

次の組み合わせは、用途が衝突するため同じ主画面に密集させない。

- CSV 取込フォームと長いレポート本文
- 不備一覧の高密度テーブルと月次レポート本文
- プライバシー抑制の説明と抑制前データの詳細
- AI assistant の会話履歴と大量の issue table
- 再確認比較と通常の実行開始フォーム

同じ run に関係する情報であっても、利用者の作業単位に合わせて分ける。トップ画面では、ready / partial / blocked、issue 件数、成果物の有無、次に確認すべき画面への導線だけを表示する。

## 4. 表示順

### 4.1 基本方針

店舗、部署、雇用区分などの一覧表示とレポート表示では、利用者がマスタ上の項目を先に確認できるようにする。表示順は固定ロジックではなく、各マスタが持つ `display_order` を正とする。`display_order` は顧客ごとのマスタ設定として扱い、コード内に個別顧客の並び順をハードコードしない。

| 対象 | 並び順 |
| --- | --- |
| マスタ登録済み項目 | 先に表示し、`display_order` 昇順で並べる |
| `display_order` 未設定の登録済み項目 | `display_order` 設定済みの登録済み項目の後ろに表示する |
| 無効化済み項目 | 原則非表示。ただし過去データ参照時は末尾に表示する |
| 未登録値、未照合、その他、推定項目 | 登録済み項目の後ろに表示する |
| 同一 `display_order` または未設定内 | コード昇順、次に名称昇順 |

コード昇順、名称昇順は主たる表示順ではなく、同順位時の安定化に使う。コードが空または欠損している場合は、同一グループ内の末尾に置き、名称昇順で並べる。名称も欠損している場合は、内部 ID 昇順で安定化する。

この並び順は UI 表示仕様の正とし、マスタ仕様側の定義は `DATA-DESIGN.md` の店舗、部署、雇用区分マスタ表示順に従う。

### 4.2 適用対象

未決事項 `OPEN-004` の対象は次の通りとする。

| 対象 | 対応マスタ |
| --- | --- |
| 店舗 | 店舗マスタ |
| 部署 | 部署マスタ |
| 雇用区分 | 雇用区分マスタ |
| 無効化済み項目 | 対応マスタに存在するが、`is_active = false` または有効期間外の項目 |
| 未登録値 | 対応マスタなし。入力値または正規化値を保持する |
| 同順位 | 同一マスタ内で `display_order` が同じ値、または `display_order` が未設定の複数項目 |

| 表示対象 | 適用 |
| --- | --- |
| 店舗別一覧 | 店舗マスタの `display_order` 昇順、次に `store_id`、`store_name` 昇順 |
| 部署別一覧 | 部署マスタの `display_order` 昇順、次に `department_id`、`department_name` 昇順 |
| 雇用区分別一覧 | 雇用区分マスタの `display_order` 昇順、次に `employment_type` コード、表示名昇順 |
| 無効化済み項目 | 原則非表示。ただし過去データ参照時は末尾に表示し、同一グループ内は `display_order` 昇順、次にコード、名称昇順で安定化する |
| 未登録値 | 登録済みマスタ値の後ろに表示し、同一グループ内は入力値または正規化値の昇順で安定化する |
| 同順位 | エラーにせず、コード昇順、次に名称昇順で安定化する |
| 集計表の行順 | 同じ並び順を使う |
| フィルタ選択肢 | 同じ並び順を使う |

### 4.3 抑制行の扱い

少人数セル、個人推測リスク、非表示理由を示す行は、元の対象項目の位置に対応させる。ただし、抑制対象の詳細な人数や属性組合せが個人推測につながる場合は、登録済み項目の詳細行としては表示せず、抑制サマリーにまとめる。

## 5. 未決事項との対応

| ID | 決定内容 | 閉じ方 |
| --- | --- | --- |
| OPEN-004 | 店舗別、部署別、雇用区分別の優先表示順は、固定ロジックではなくマスタの `display_order` を使う。対象は店舗マスタ、部署マスタ、雇用区分マスタ、無効化済み項目、未登録値とする。無効化済み項目は原則非表示とし、過去データ参照時だけ末尾表示する。未登録値、未照合、その他、推定項目は登録済み項目の後ろに置き、同順位時はコード昇順、次に名称昇順で安定化する。 | UI 仕様とマスタ仕様に反映済み。Lean のブロッカーではない。 |
| UI-STACK-001 | local UI は Tauri + Vite + React + TypeScript、shadcn/ui、TanStack Table、TanStack Query、assistant-ui で実装する。Ollama assistant は UI から直接呼ばず Local Server API 経由にする。 | UI 技術選定として反映済み。安全境界は `ARCHITECTURE.md` の UI / Guide AI 境界に従う。 |
| UI-LAYOUT-001 | local UI は一画面に全情報を詰め込まず、実行、結果、不備一覧、レポート、再確認、ガイド、設定をページ単位で分ける。詳細はタブ、ドロワー、別タブ表示を使う。 | 画面分割方針として反映済み。初期画面は全情報ではなく次に確認すべき場所へ進めるサマリーにする。 |
| UI-DIST-001 | 最終配布形態は Tauri + React の installer 付き desktop app とする。初期開発は browser + local server で進めてもよいが、本番配布では利用者がインストール済みアプリから起動できる形にする。 | Desktop 配布方針として反映済み。Tauri は配布と起動管理を担当し、業務ロジックと AI 安全境界は Local Server API 側に置く。 |
| UI-DIST-002 | PostgreSQL、Ollama、初期モデル `qwen3:8b` は installer 同梱を目標にする。利用者が別途手動インストールしなくても初回起動できる構成を優先する。 | 同梱配布方針として反映済み。ライセンス、サイズ、更新、データディレクトリ、モデル更新方法は implementation / operations で検証する。 |

# 本番ロードマップ

Date: 2026-06-01
Status: draft
Aligned with: `docs/product/REQUIREMENTS.md`

## フェーズ 0: リポジトリの立ち上げ

Outcome: `C:\Users\kinbo\labor-lens` に clean Rust workspace と docs contract が存在する。

- 最初の scaffold が承認されたら Git を初期化する。
- Rust workspace と crate boundaries を作成する。
- formatting、linting、test commands を追加する。
- fixture folders と generated outputs の policy を追加する。
- local server、local DB、background job を不可逆な architecture choice として ADR 化する。

## フェーズ 1: データ品質コアとローカルDB

Outcome: production Rust が selected CSV inputs を読み取り、ローカルDBに run、dataset、issue、artifact metadata を保存できる。

- domain value objects と issue codes を実装する。
- local DB schema と migration の最小形を追加する。
- employees and attendance ingest を先に実装する。
- configuration 経由で Japanese header mapping を追加する。
- raw input、normalized data、issues を区別して保存する。
- `run_summary.json`、`issues.csv`、`profile_report.json` を出力する。
- valid and invalid fixtures 用の golden tests を追加する。

## フェーズ 2: ローカルサーバーとバックグラウンドジョブ

Outcome: local server が CSV 取り込み、検査、集計、レポート生成を background job として実行できる。

- local server の最小 API を追加する。
- job start、job progress、job result、artifact retrieval を実装する。
- heavy CSV processing を UI 操作から分離する。
- deterministic output と raw-input immutability checks を追加する。
- job logs、run IDs、diagnostic metrics を追加する。

## フェーズ 3: 全データセット対応

Outcome: production engine が 5 つの key dataset families を local DB 経由で扱える。

- date and time-slot support を持つ sales を追加する。
- explicit grain support を持つ labor costs を追加する。
- privacy-first 伏せ字処理を持つ fatigue input を追加する。
- join-readiness reports を追加する。
- aggregate metrics を追加する。
- legacy schema と target schema を区別する。

## フェーズ 4: 10000人規模・3年分の性能検証

Outcome: 10000人 × 3年分の勤怠データを想定した scale fixture を取り込み、検査、集計、レポート生成できる。

- scale fixture generator を追加する。
- 約1095万行の勤怠データを想定した performance smoke を追加する。
- memory usage、processing time、DB size、artifact size を測定する。
- 必要に応じて streaming、chunking、indexing、incremental aggregation を導入する。
- performance result を release gate として記録する。

## フェーズ 5: ローカル確認 UI

Outcome: ユーザーが local UI から job を開始し、進捗と結果を確認できる。

- local server に接続する UI を作成する。
- dataset summary、issue list、join readiness、report details を表示する。
- calculations は UI ではなく local server job に置く。
- UI smoke checks を追加する。
- ローカルデモ用の架空サンプルデータ導線を追加する。

## フェーズ 6: レポートとローカル配布

Outcome: generated reports が practical review and sharing に使える状態になる。

- Markdown/CSV/JSON report contracts を安定させる。
- PDF または rich HTML rendering に Python helper が必要か判断する。
- logs and diagnostic export を追加する。
- local server、local DB、UI の起動・停止手順を整備する。
- core、jobs、storage、UI behavior が安定した後に Windows packaging を検討する。

## フェーズ 7: ローカルアシスタント

Outcome: LaborLens が reports and constraints をローカルで説明できる。

- product guides and report definitions 用の document indexing を追加する。
- narrow adapter を通じて Ollama を統合する。
- answers で local documentation を引用する。
- prompt-injection and privacy tests を追加する。
- Japanese and English answers を別々に評価する。

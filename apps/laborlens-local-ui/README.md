# laborlens-local-ui

LaborLens の local UI 境界。

予定している責務:

- input dataset を選択する。
- server 経由で local run を開始する。
- progress、readiness、issues、reports を表示する。
- database、source archive、suppressed internal dataset へ直接 access しない。

現在の実装は、Vite + React + TypeScript の local UI である。`/api/use-cases` を呼んで 14 個のユースケースボタンを描画し、各ボタンから `/api/use-cases/{use_case_id}/sample-data` を読み込む。

UI は PostgreSQL や source archive へ直接 access しない。1000 人分の架空日本人従業員 seed は local server API 経由で受け取り、CSV 検査、joinability、privacy suppression、report generation は UI に再実装しない。

CSV run 用の `/api/runs` 呼び出し欄は残しているが、HTTP endpoint と worker 接続は後続 slice の対象である。

## Development

local server を別 terminal で起動してから UI を起動する。

```powershell
cargo run -p laborlens-local-server
cd apps/laborlens-local-ui
npm install
npm run dev
```

Vite dev server は `http://127.0.0.1:1420` で起動し、`/api` を既定で `http://127.0.0.1:5174` へ proxy する。local server の port を変える場合は `LABORLENS_LOCAL_SERVER_URL` を設定する。

Tauri shell は `src-tauri/` に最小構成だけを置いている。local server / worker の sidecar 起動管理はまだ接続していない。

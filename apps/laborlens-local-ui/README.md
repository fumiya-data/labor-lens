# laborlens-local-ui

LaborLens の local UI 境界。

予定している責務:

- input dataset を選択する。
- server 経由で local run を開始する。
- progress、readiness、issues、reports を表示する。
- database、source archive、suppressed internal dataset へ直接 access しない。

現在の実装は、`index.html`、`src/app.js`、`src/styles.css` の静的 UI である。`/api/use-cases` を呼んで 14 個のユースケースボタンを描画し、各ボタンから `/api/use-cases/{use_case_id}/sample-data` を読み込む。

UI は PostgreSQL や source archive へ直接 access しない。1000 人分の架空日本人従業員 seed は local server API 経由で受け取り、CSV 検査、joinability、privacy suppression、report generation は UI に再実装しない。

CSV run 用の `/api/runs` 呼び出し欄は残しているが、HTTP endpoint と worker 接続は後続 slice の対象である。

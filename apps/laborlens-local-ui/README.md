# laborlens-local-ui

LaborLens の local UI 境界。

予定している責務:

- input dataset を選択する。
- server 経由で local run を開始する。
- progress、readiness、issues、reports を表示する。
- database、source archive、suppressed internal dataset へ直接 access しない。

現在の初期実装は、`index.html`、`src/app.js`、`src/styles.css` の静的 UI である。`/api/runs` を呼び、server から返る progress と artifact list を表示する。CSV 検査、joinability、privacy suppression、report generation は UI に再実装しない。

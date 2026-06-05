# laborlens-local-server

LaborLens の local server 境界。

予定している責務:

- UI command を受け取る。
- `RunId` を発行し追跡する。
- background job を登録する。
- job progress と generated artifact を公開する。
- UI access を application safety boundary の内側に保つ。

現在の実装は Rust crate `laborlens-local-server` である。`LocalServer::start_run` で `apps/laborlens-rust` の `run_ingest_workflow` を呼び、run creation、job progress、artifact listing の contract を固定している。

あわせて、ローカル UI 用の小さな HTTP server を持つ。次の API を公開する。

- `GET /api/use-cases`: `docs/product/USE-CASES.md` に対応する 14 個のユースケースボタン定義を返す。
- `GET /api/use-cases/{use_case_id}/sample-data`: 1000 人分の架空日本人従業員 seed を持つ `laborlens.demo_employees` 相当の repository から、ユースケース別サンプルを返す。
- `POST /api/runs`: 既存の run contract の後段接続用。現時点の HTTP 実行では未接続である。

起動例:

```powershell
cargo run -p laborlens-local-server
```

既定では `127.0.0.1:5174` から順に空きポートを探し、`apps/laborlens-local-ui` を配信する。

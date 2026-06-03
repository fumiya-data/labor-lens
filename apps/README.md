# アプリケーション

実行可能なアプリケーション境界を置く directory。

- `laborlens-rust/`: 単一 Rust modular monolith workspace。本番 Rust behavior はここから開始し、bounded context module で整理する。
- `laborlens-local-server/`: local API、job orchestration、local database access、artifact serving。
- `laborlens-local-ui/`: run setup、progress、issue list、report viewing のための local UI。

server と UI の directory は、各 project を初期化するまで delivery boundary の placeholder として残す。business rule を再実装せず、Rust monolith contract を呼び出す。

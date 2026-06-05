# ツール

fixture 生成、ローカル検証、レポート後処理、リポジトリ保守に使う補助スクリプトを置く。

ツールは小さく明示的に保つ。本番 Rust ロジックは `apps/laborlens-rust` に置き、Python レポートツール群は薄い層として、抑制済み report artifact だけを消費する。

- `validate-repository-structure.ps1`: modular monolith scaffold、docs 参照、Lean Phase 1 の path、reports/Python 接続方針を検証する。
- `validate-db-schema.ps1`: PostgreSQL migration、DB interface 文書、Rust DB command model の静的整合性を検証する。
- `seed-demo-db.ps1`: PostgreSQL に schema を適用し、`laborlens.demo_employees` へ 1000 人分の架空日本人従業員 seed を投入する。
- `generate-scale-fixture.ps1`: 固定 seed 方針に従い、scale fixture 用の合成 employees/attendance CSV を生成する。

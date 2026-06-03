# レポート

レポート例と生成済み出力の参照ファイルを置く。

- `examples/`: 文書化とレビューに使う安定した例。
- `generated/`: ローカル生成物。Git では無視する。
- `report_app/`: Pike が担当する Python レポート出力アプリ。

Pike の Python report app は、Rust monolith が出力する抑制済み公開 artifact に接続する。

現在の renderer 入力:

- `cargo run -p laborlens-rust` が出力する `laborlens.public_report.v1` JSON

将来の出力参照として、次の分割 artifact 名も利用できる。

- `public_report_model.json`
- `report.md`
- `issues.csv`
- `artifact_manifest.json`

Python は、原本 CSV の再読込、core analysis の再計算、privacy/safety context を通っていない内部 dataset へのアクセスを行ってはならない。

## Python report app（レポートアプリ）

report app は Python 標準ライブラリだけを使う。

保存済み public JSON fixture を Markdown に変換する:

```powershell
python reports/report_app/main.py --input reports/examples/public_report_v1.json --output reports/examples/public_report_run-smoke-001.md
```

Rust smoke contract から直接変換する:

```powershell
cargo run -p laborlens-rust --quiet | python reports/report_app/main.py --input - --output reports/examples
```

`--output -` を指定すると Markdown を stdout に出力する。`--output` が directory を指す場合、app はその中に `public_report_<run_id>.md` を書き込む。

プライバシー上の挙動:

- 入力 contract version は `laborlens.public_report.v1` でなければならない。
- payload 内のどこかに `employee_ref`、`fatigue_value`、`sleep_duration_hours`、`fatigue_comment` という JSON key がある場合、入力は拒否される。
- Markdown は group profile と suppression metadata を含む公開 aggregate field だけから生成する。

## 将来の renderer hook

PDF、HTML、print layout、chart renderer は初期スコープ外である。Markdown/public-contract boundary の後段に renderer hook として追加し、同じ検証済み public JSON または Markdown output を消費させる。raw CSV や DB data に直接接続してはならない。

# LaborLens 実装計画

日付: 2026-06-03
状態: initial implementation plan
backlog 出典: `IMPL-BACKLOG.md` Highest Priority
repository plan 文書: `docs/planning/REPOSITORY-PLAN.md`
DB interface 文書: `docs/planning/DB-INTERFACES.md`
Rust 入口: `apps/laborlens-rust`

## 0. 目的

この文書は、LaborLens の次の実装 slice を、担当境界、TDD sequence、review unit、acceptance check まで落とし込むための実装計画である。

次に進める Highest Priority slice は次の通りとする。

```text
employees/attendance CSV ingest
  -> DB run/job registration
  -> issues and run_summary artifact generation
```

この slice は UI を始める前に、Rust ingest、PostgreSQL state、issue 出力、run summary traceability、Python report connection を小さく接続することを目的にする。

## 1. 現在の前提

既にある前提:

- Rust implementation entry は `apps/laborlens-rust` である。
- Rust 側は modular monolith として、`ingest`、`workforce_analysis`、`privacy_safety`、`reporting`、`guidance` context を持つ。
- Radomil は `privacy_safety` と `reporting` の最初の public report contract を作成済みである。
- Dabian は PostgreSQL schema、`docs/planning/DB-INTERFACES.md`、`tools/validate-db-schema.ps1` を作成済みである。
- Leonard は Lean privacy / aggregation privacy / RunArtifact traceability の build を維持している。
- Pike は `laborlens.public_report.v1` を読む Python Markdown renderer を作成済みである。

この計画では、上記の成果を revert しない。`IMPL-BACKLOG.md` は main 側で必要に応じて更新する。

## 2. slice の適用範囲

### 2.1 対象範囲

この slice で扱うこと:

- employees CSV と attendance CSV の最小 ingest。
- 日本語 header mapping の最小 contract。
- 原本 CSV の source hash と input ref 生成。
- schema issue と data-quality issue の最小生成。
- PostgreSQL への run registration と job registration。
- `DB-INTERFACES.md` に従う write order と table boundary。
- `issues` と `run_summary` の artifact generation。
- `artifact_manifest` または `report_artifacts` への suppressed artifact metadata 登録。
- CLI smoke で、UI を介さずに一連の処理を確認する。

### 2.2 対象外

この slice では扱わないこと:

- local server と local UI の本格実装。
- workforce analysis の full readiness / joinability / labor-cost analysis。
- fatigue、sales、labor-cost、shift、leave datasets の本格 ingest。
- small-group suppression の本格実装。Lean / Rust の既存方針との接続確認までに留める。
- PDF、HTML、chart の本格 renderer。
- raw CSV rows の DB 永続化。
- PostgreSQL partitioning。

## 3. agent ごとの所有範囲

| agent | 主責務 | 所有してはならないもの |
| --- | --- | --- |
| Fred | `IMPLEMENTATION-PLAN.md`、workflow alignment、review unit と acceptance check の定義。構造検証と計画文書の整合確認。 | Rust/DB/Lean/Python 実装、`IMPL-BACKLOG.md` 更新。 |
| Radomil | `apps/laborlens-rust` の Rust implementation。employees/attendance ingest、header mapping、issue generation、run_summary artifact contract、CLI smoke。 | PostgreSQL schema 変更の最終判断、Python rendering、Lean proof。 |
| Dabian | PostgreSQL interface。`DB-INTERFACES.md`、migration、`validate-db-schema`、DB adapter contract、static SQL validation。 | Rust domain rule、Python report rendering、Lean theorem。 |
| Leonard | Lean build と theorem alignment。RunArtifact、privacy suppression、source preservation / joinability theorem との対応確認。 | Rust ingest implementation、PostgreSQL migration、Python renderer。 |
| Pike | Python report tests と renderer connection。Rust が出す suppressed artifacts だけを読み、Markdown / future renderer hooks を確認する。 | raw CSV 読取、PostgreSQL 直接読取、core analysis の再計算。 |

## 4. PostgreSQL interface rule

PostgreSQL interface は `docs/planning/DB-INTERFACES.md` を正とする。

Radomil と Dabian は、少なくとも次の write order を実装計画上の contract として扱う。

1. `run_records`
2. `input_refs`
3. `jobs`
4. `normalized_refs`
5. `policy_refs`
6. `issues`
7. `privacy_suppressions`
8. `output_refs`
9. `audit_refs`
10. `run_artifacts`
11. `artifact_manifests` and `report_artifacts`

この slice の実装では、Rust domain logic を SQL trigger に入れない。SQL は system of record と integrity boundary を担当し、header mapping、issue classification、public report contract は Rust 側の application/domain code で扱う。

## 5. TDD sequence

実装は次の順に進める。各 step は先に test / validation を追加し、失敗を確認してから実装する。

### 5.1 fixture test

担当: Radomil。レビュー: Fred。

最初に employees/attendance fixtures を定義する。

- `fixtures/valid`: minimal employees CSV、minimal attendance CSV。
- `fixtures/invalid`: required header missing、unknown Japanese header、invalid date/time、missing employee id。
- `fixtures/privacy`: public output に raw sensitive value が出ないことを確認するための補助入力。

受け入れ条件:

- fixture 名、dataset kind、expected issue code、expected readiness が明示されている。
- 原本 fixture はテスト実行中に変更されない。
- source hash を期待値として比較できる。

### 5.2 Rust unit test

担当: Radomil。

`apps/laborlens-rust` に failing Rust unit tests を先に追加する。

対象例:

- employees CSV の必須 header を検出できる。
- attendance CSV の日本語 header を標準 field へ map できる。
- 欠落 header を schema issue にできる。
- invalid date/time を data-quality issue にできる。
- source hash と input ref が `run_summary` へ渡る。
- public artifact に employee name、raw fatigue value、sleep duration、fatigue comment が出ない。

コマンド:

```powershell
cargo test -p laborlens-rust
```

受け入れ条件:

- 実装前に test が失敗する。
- implementation 後に `cargo test -p laborlens-rust` が成功する。
- ingest context と reporting context の境界が保たれ、UI や Python へ core logic が流出しない。

### 5.3 DB adapter test と静的 SQL 検証

担当: Dabian と Radomil。

DB adapter は `DB-INTERFACES.md` の table policy と write order に従う。ローカル PostgreSQL integration test がすぐに安定しない場合は、次の二段階で始める。

1. static SQL validation: migration と interface doc が整合していることを確認する。
2. repository trait / adapter tests: Rust 側で expected write operation order と payload shape を検証する。

コマンド:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\validate-db-schema.ps1
```

受け入れ条件:

- `validate-db-schema` が成功する。
- `run_records`、`input_refs`、`jobs`、`issues`、`run_artifacts`、`artifact_manifests`、`report_artifacts` の interface が実装対象として明示されている。
- `jobs.state` は `queued`、`running`、`retry_wait`、`succeeded`、`failed`、`cancel_requested`、`canceled` の範囲に収まる。
- raw CSV rows または sensitive values を public/report metadata table に保存しない。

### 5.4 Lean build

担当: Leonard。

この slice で Lean file を変更する必要がある場合は Leonard が担当する。Radomil / Dabian は、Lean の `RunArtifact`、privacy suppression、source preservation に対応する実装 contract 名を壊さない。

コマンド:

```powershell
Push-Location lean
lake build
Pop-Location
```

受け入れ条件:

- `lake build` が成功する。
- RunArtifact traceability と DB `run_artifacts` / artifact manifest の対応が説明できる。
- source hash を保持する実装が、将来の source preservation theorem へ接続できる。

### 5.5 Python report test

担当: Pike。

Pike は Rust が生成する suppressed public report artifacts を読む。Python は raw CSV と PostgreSQL を直接読まない。

コマンド:

```powershell
python -m unittest discover reports/report_app/tests
```

受け入れ条件:

- Python tests が成功する。
- `public_report_model.json` または current `laborlens.public_report.v1` JSON から Markdown を生成できる。
- `issues` と `run_summary` の情報を表示できる。
- forbidden raw keys を含む入力は拒否される。

### 5.6 integration smoke

担当: Radomil。レビュー: Fred、Dabian、Pike。

最初の integration smoke は local server / UI なしで行う。

期待する flow:

```text
fixtures employees/attendance
  -> Rust ingest
  -> RunId and source hashes
  -> DB run/job registration or adapter smoke
  -> issues
  -> run_summary
  -> suppressed public report JSON
  -> Python Markdown renderer
```

候補コマンド:

```powershell
cargo test -p laborlens-rust
powershell -NoProfile -ExecutionPolicy Bypass -File tools\validate-db-schema.ps1
python -m unittest discover reports/report_app/tests
cargo run -p laborlens-rust --quiet | python reports/report_app/main.py --input - --output -
```

受け入れ条件:

- smoke run に `RunId` がある。
- `run_summary` に input refs、source hashes、issue count、artifact refs がある。
- `issues` は行、列、理由、優先度、関連 RunId または input hash を持つ。
- DB job state が queued -> running -> succeeded or failed のどちらかに説明可能に遷移する。
- Python renderer は suppressed artifact だけを読む。
- raw CSV content、employee name、personal fatigue value、sleep duration、fatigue comment が public report に出ない。

## 6. review unit

| review unit | 担当 | 範囲 | 必須 check |
| --- | --- | --- | --- |
| RU-1 Implementation plan | Fred | この文書、workflow status | required keyword rg、repository structure validation |
| RU-2 Fixtures and expected issues | Radomil | employees/attendance fixture set、expected issue table | fixture tests、raw input unchanged check |
| RU-3 Rust ingest unit | Radomil | header mapping、parsing、hash/input ref、issue generation | `cargo test -p laborlens-rust` |
| RU-4 DB interface adapter | Dabian / Radomil | write order、job state、RunArtifact refs | `validate-db-schema`、adapter tests or static SQL validation |
| RU-5 Reporting artifacts | Radomil / Pike | issues、run_summary、artifact manifest、Markdown render | `cargo test -p laborlens-rust`、Python unittest |
| RU-6 Cross-agent smoke | Fred / main | Rust + DB validation + Lean + Python smoke | cargo、validate-db-schema、lake build、Python tests、pipe smoke |

## 7. slice の受け入れ check

この slice は、次の check がすべて true のとき acceptable とする。

- employees/attendance CSV ingest を `apps/laborlens-rust` から実行できる。
- 必須の日本語 headers が安定した internal fields へ map される。
- 欠落または invalid fields が schema/data-quality issues を生成する。
- source file hash が capture され、input refs が `run_summary` で利用可能である。
- PostgreSQL registration が `DB-INTERFACES.md` に従う。
- `run_records`、`input_refs`、`jobs`、`issues`、RunArtifact-related refs が表現されている。
- `issues` と `run_summary` artifacts が deterministic に生成される。
- `cargo test -p laborlens-rust` が成功する。
- `powershell -NoProfile -ExecutionPolicy Bypass -File tools\validate-db-schema.ps1` が成功する。
- `lean/` で `lake build` が成功する。
- `python -m unittest discover reports/report_app/tests` が成功する。
- integration smoke で Rust public output を Pike の Python renderer に渡せる。
- raw または sensitive values が public/report artifacts へ漏れない。

## 8. backlog 引き継ぎ

この plan 作成後、次の backlog work は次の順で進める。

1. Radomil が failing fixture と Rust ingest tests を書く。
2. Radomil が `apps/laborlens-rust` に employees/attendance ingest を実装する。
3. Dabian と Radomil が DB adapter behavior を `DB-INTERFACES.md` に接続する。
4. Radomil が real ingest output 用の issue と run_summary generation を拡張する。
5. artifact shape が変わる場合、Pike が renderer tests を更新する。ただし raw CSV や DB を直接読まない。
6. Leonard が Lean build を green に保ち、theorem-name alignment の問題があれば記録する。
7. Fred または main agent が cross-agent verification commands を実行し、結果を記録する。

`IMPL-BACKLOG.md` は main 側で後から更新する。

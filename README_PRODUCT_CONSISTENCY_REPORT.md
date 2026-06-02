# README と docs/product の整合性確認レポート

Date: 2026-06-02
Scope: `README.md` と `docs/product/` 配下の Markdown

## 結論

README と `docs/product/` の内容は、現状では完全には整合していない。

主な原因は、`docs/product/` 側の文書が増え、要求・業務ルール・受け入れ基準・データ設計・アーキテクチャ設計まで進んでいる一方で、README が初期状態の説明を多く残していることである。

## 確認した product 文書

`docs/product/` 配下には、現時点で次の文書がある。

- `ACCEPTANCE-CRITERIA.md`
- `ARCHITECTURE.md`
- `BUSINESS-RULES.md`
- `DATA-DESIGN.md`
- `GLOSSARY.md`
- `LEAN-SPEC-PLANNING.md`
- `REPOSITORY-PLAN.md`
- `REQUIREMENTS.md`
- `USE-CASES.md`
- `WORKFLOW.md`

## 不整合 1: README の計画文書一覧が古い

README の「計画文書」は、次の 2 件だけを挙げている。

- `README.md:39` `docs/product/WORKFLOW.md`
- `README.md:40` `docs/product/REPOSITORY-PLAN.md`

しかし、`docs/product/` には要求仕様、用語集、業務ルール、受け入れ基準、データ設計、アーキテクチャ、Lean 仕様化計画も存在する。

影響:

- README だけを読むと、現在作成済みの主要仕様文書が見つけにくい。
- `docs/product/` の現状より、README が初期段階のままに見える。

推奨:

- README の「計画文書」を、現在の `docs/product/` 一覧に合わせて更新する。
- 各文書の役割を `WORKFLOW.md` の文書責務と合わせる。

## 不整合 2: ローカルガイド AI / RAG の位置づけが異なる

README では、ローカルアシスタント / RAG は「コアワークフローの安定後に、Ollama ベースのローカル実行環境を検討する」とされている。

- `README.md:35`

一方、product 文書では、ローカル使い方ガイド AI は要求・業務ルール・アーキテクチャ・受け入れ基準の対象になっている。

- `docs/product/REQUIREMENTS.md:102` ローカル使い方ガイド AI を対象機能に含めている。
- `docs/product/REQUIREMENTS.md:141` ガイド AI もローカル実行を基本とするとしている。
- `docs/product/REQUIREMENTS.md:413` 以降でローカル使い方ガイド AI を仕様化している。
- `docs/product/ARCHITECTURE.md:62` から `docs/product/ARCHITECTURE.md:66` で、ガイド AI と RAG を初期構成に含めている。
- `docs/product/ARCHITECTURE.md:160` で、Ollama `qwen3:8b` を初期モデルとする ADR が置かれている。

影響:

- README は「将来検討」に見えるが、product 文書では「安全制約つきの製品機能」として扱っている。
- README のスタック表が、現在のアーキテクチャ決定を反映していない。

推奨:

- README では「将来検討」ではなく、「ローカルガイド AI は計画対象。ただし抑制後データと根拠文書だけを参照する」という表現に寄せる。
- 初期モデルを README に書く場合は、`docs/product/ARCHITECTURE.md` の `qwen3:8b` と合わせる。

## 不整合 3: README のデータ前提がデモと製品本体を混同して見える

README は、このリポジトリを「架空またはサンプルの業務データを前提」と説明している。

- `README.md:8`

product 文書では、製品本体はローカル環境で業務 CSV を取り込むアプリケーションとして定義され、架空データ限定はローカルデモに対する制約として書かれている。

- `docs/product/REQUIREMENTS.md:42` LaborLens は勤怠、人件費、売上、従業員マスタ、疲労関連データなどの CSV をローカル環境で取り込む。
- `docs/product/REQUIREMENTS.md:466` ローカルデモ版は架空のサンプルデータだけを使う。
- `docs/product/REQUIREMENTS.md:470` ローカルデモは実在個人情報を含まない架空データだけを扱う。
- `docs/product/USE-CASES.md:24` から `docs/product/USE-CASES.md:25` でも、ローカルデモ版では実在データを扱わないと説明している。

影響:

- README だけ読むと、製品本体もサンプルデータ専用に見える。
- product 文書上の「実務データをローカルで扱う本番アプリ」と「ポートフォリオ用デモは架空データ限定」の区別が弱くなる。

推奨:

- README では、製品本体の想定とローカルデモのデータ制約を分けて書く。
- 例: 「製品仕様は実務 CSV のローカル処理を想定する。ポートフォリオ用デモと検証データは架空データに限定する。」

## 不整合 4: README の入力データ範囲が product 文書より狭い

README の製品説明は、勤怠・疲労・人件費・売上データを中心にしている。

- `README.md:6`

product 文書では、それに加えて従業員マスタ、シフトデータ、休暇情報、共有予定データも入力データとして扱っている。

- `docs/product/REQUIREMENTS.md:162` から `docs/product/REQUIREMENTS.md:168`
- `docs/product/DATA-DESIGN.md:129` から `docs/product/DATA-DESIGN.md:132`

影響:

- README の製品スコープが、現在の要求仕様より小さく見える。
- 外部共有前チェックや有給休暇、シフト粒度の設計が README から読み取れない。

推奨:

- README の製品説明または設計上の制約に、従業員マスタ、シフト、休暇、共有予定データを追加する。
- ただし README は概要にとどめ、詳細は `docs/product/REQUIREMENTS.md` と `docs/product/DATA-DESIGN.md` へ誘導する。

## 不整合 5: ローカル DB の決定状態が product 文書内で揺れている

README はローカル DB とだけ書き、具体的な DB エンジンを明示していない。

- `README.md:28`

一方で product 文書内では、DB 方針が揺れている。

- `docs/product/ARCHITECTURE.md:62` ローカル DB は PostgreSQL。
- `docs/product/ARCHITECTURE.md:158` PostgreSQL 採用を ADR として記録している。
- `docs/product/ARCHITECTURE.md:334` ローカル DB には PostgreSQL を採用すると明記している。
- `docs/product/DATA-DESIGN.md:25` SQLite、DuckDB、PostgreSQL embedded 相当のいずれでも実現できるようにするとしている。
- `docs/product/REPOSITORY-PLAN.md:134` SQLite か DuckDB などを未決事項としている。

影響:

- README を現在の product 文書に合わせる際、PostgreSQL を確定方針として書くべきか判断しにくい。
- `ARCHITECTURE.md` を正とするなら、`DATA-DESIGN.md` と `REPOSITORY-PLAN.md` が古い。

推奨:

- DB 方針の正を決める。現状では `ARCHITECTURE.md` が最も具体的なので、これを正にするなら README と他 product 文書を PostgreSQL 方針へ合わせる。
- まだ未決に戻すなら、`ARCHITECTURE.md` の ADR を弱める。

## 不整合 6: `WORKFLOW.md` の作成状態が現状と合っていない

`docs/product/WORKFLOW.md` は、実在する文書をまだ「未作成」としている。

- `docs/product/WORKFLOW.md:75` `GLOSSARY.md` が未作成
- `docs/product/WORKFLOW.md:76` `BUSINESS-RULES.md` が未作成
- `docs/product/WORKFLOW.md:77` `ACCEPTANCE-CRITERIA.md` が未作成
- `docs/product/WORKFLOW.md:79` `DATA-DESIGN.md` が未作成
- `docs/product/WORKFLOW.md:80` `ARCHITECTURE.md` は一部作成済み

しかし、これらのファイルは `docs/product/` に存在する。

影響:

- README の計画文書一覧を更新しても、参照先のワークフロー文書が古い状態を示す。
- 文書の進捗が読者に誤って伝わる。

推奨:

- `docs/product/WORKFLOW.md` の状態欄を、現在のファイル実在状況と文書内容に合わせて更新する。
- 「未作成」と「draft」は分けて表現する。

## 不整合 7: product 文書に古い参照名が残っている

product 文書内に、現在存在しないと思われる古い文書名への参照が残っている。

- `docs/product/ACCEPTANCE-CRITERIA.md:5` `Source: REQUIREMENTS_BRUSHED.md`
- `docs/product/BUSINESS-RULES.md:6` `Source: REQUIREMENTS_BRUSHED.md`
- `docs/product/BUSINESS-RULES.md:21` `REQUIREMENTS_BRUSHED.md`
- `docs/product/DATA-DESIGN.md:10` `REQUIREMENTS_BRUSHED.md`
- `docs/product/LEAN-SPEC-PLANNING.md:7` `docs/product/REQUIREMENTS_BRUSHED.md`
- `docs/product/LEAN-SPEC-PLANNING.md:31` `REQUIREMENTS_BRUSHED.md`
- `docs/product/WORKFLOW.md:7` `docs/development/WORKFLOW.md`
- `docs/product/REQUIREMENTS.md:8` `docs/development/WORKFLOW.md`

現状のファイル一覧では、`docs/product/REQUIREMENTS.md` と `docs/product/WORKFLOW.md` は存在するが、`REQUIREMENTS_BRUSHED.md` と `docs/development/WORKFLOW.md` は確認できない。

影響:

- README から product 文書へ読者を誘導しても、参照元・参照先が古く見える。
- 文書間のトレーサビリティが弱くなる。

推奨:

- `REQUIREMENTS_BRUSHED.md` は `REQUIREMENTS.md` に置き換える。
- `docs/development/WORKFLOW.md` は、現状の正が `docs/product/WORKFLOW.md` ならそこへ置き換える。
- 旧文書を履歴として残したい場合は、ADR または移行メモで明示する。

## README 側で整合している点

次の点は、README と product 文書で大きな矛盾は見つからなかった。

- 原本 CSV を変更しない方針。
- ローカルサーバー、ローカル DB、バックグラウンドジョブ、ローカル UI を組み合わせる方針。
- 10000 人規模・3 年分の勤怠データを設計目標にする方針。
- データ品質 issue と業務上の推奨または確認ポイントを分離する方針。
- 個人疲労値をユーザー向け出力に出さない方針。

## 対応優先度

1. README の「計画文書」一覧を現状の `docs/product/` に合わせる。
2. ローカルガイド AI / RAG の位置づけを、product 文書の要求・アーキテクチャに合わせる。
3. README の「架空またはサンプルデータ」表現を、製品本体とローカルデモで分ける。
4. DB 方針について、`ARCHITECTURE.md` を正にするか未決に戻すか決める。
5. `docs/product/WORKFLOW.md` の状態欄と古い参照名を更新する。

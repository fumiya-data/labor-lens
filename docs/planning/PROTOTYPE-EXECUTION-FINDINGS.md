# 試作品実行から得た知見

Date: 2026-06-01
Status: imported from sub-agent read-only review
Aligned with: `docs/product/REQUIREMENTS.md`

このメモは、`C:\Users\kinbo\labor-lens-prototype` にある試作品実行出力から、本番製品に影響する点をまとめる。

## 知見

### 1. Smoke の成功範囲は狭い

最新の source smoke path は小さな正常系 sample では通っているが、対象は 27 行だけである。一方、raw-data profile は 3927 行を対象にし、2 件の fatal issues と 276 件の errors を報告している。

本番への示唆: sample smoke だけでは release confidence として不十分である。production test suite には、valid fixtures、invalid fixtures、raw-schema fixtures、より大きな performance fixtures が必要である。

特に現在の product spec では、10000人規模・3年分を設計目標にしている。勤怠だけでも約1095万行を想定するため、小さな smoke と大規模 scale smoke を分けて扱う。

### 2. 売上スキーマにずれがある

raw `sales.csv` は monthly-sales oriented である。一方、productized sample は date と time-slot fields を期待している。raw run は `sales_date` と `time_slot` の missing を報告している。

本番への示唆: sales schemas は versioned にする必要がある。製品は legacy monthly sales input と target date/time-slot sales input を区別すべきである。その方法は、明示的な schema selection または migration layer のどちらかでよい。

### 3. 人件費の粒度が入力間で安定していない

productized sample には labor-cost grain column が含まれるが、raw `labor_costs.csv` には含まれていない。raw run は labor-cost rows 全体で `cost_grain` errors を報告している。

本番への示唆: production schema は labor-cost grain を明示する必要がある。legacy inputs に grain がない場合、明確な remediation message で reject するか、文書化された inference/migration rule を support する。

### 4. プライバシーに関わる出力はシステム境界として扱う必要がある

最近の smoke outputs では individual fatigue values が伏せ字になっている。しかし、古い raw outputs には fatigue score、sleep hours、comments が生成物に含まれている場合がある。

本番への示唆: privacy suppression は final rendering だけではなく、report models と UI state の前に置くべき system boundary である。生成済み artifact storage では、古い unsafe outputs が releasable outputs と混ざらないようにする。

### 5. ローカルDBとバックグラウンドジョブの検証が必要である

prototype verification report では、次の領域が未検証または完全には実行されていない。

- raw input file immutability
- deterministic repeated runs
- invalid fixture coverage
- local DB schema and migration
- background job progress and recovery
- 10000人 × 3年分を想定した scale fixture

本番への示唆: local server + local DB product-ready と扱う前に、これらの checks を release gates にするべきである。

## 本番での対応

1. `laborlens-ingest` に schema-version handling を追加する。
2. `laborlens-storage` に local DB schema と migration を追加する。
3. `laborlens-jobs` に CSV 取り込み、検査、集計、レポート生成 job を追加する。
4. raw legacy fixtures を target product fixtures とは分離して追加する。
5. duplicate IDs、unknown IDs、time reversal、negative values、missing required columns、invalid labor-cost grain、small health-related groups 用の invalid fixtures を追加する。
6. report または UI model が作られる前に privacy-by-default を強制する。
7. deterministic output tests と input-hash before/after tests を追加する。
8. 10000人 × 3年分の scale fixture と performance smoke を追加する。
9. local server、local DB、local UI workflows が安定するまで installer release gates を後回しにする。

# Fixtures（テストデータ）

合成テスト dataset を置く。

- `valid/`: 正常系 CSV 例。
- `invalid/`: schema、data-quality、joinability の失敗例。
- `privacy/`: small-group と sensitive-output suppression の例。
- `performance/`: 中規模 performance smoke fixture。
- `scale/`: 10000 人規模、複数年 scenario 用の固定 seed scale fixture。

この repository には架空データだけを保存する。

scale fixture は `fixtures/scale/scale-seed.json` を基準にし、必要時に次で生成する。

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools\generate-scale-fixture.ps1
```

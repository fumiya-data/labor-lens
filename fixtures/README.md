# Fixtures（テストデータ）

合成テスト dataset を置く。

- `valid/`: 正常系 CSV 例。
- `invalid/`: schema、data-quality、joinability の失敗例。
- `privacy/`: small-group と sensitive-output suppression の例。
- `performance/`: 中規模 performance smoke fixture。
- `scale/`: 10000 人規模、複数年 scenario 用の固定 seed scale fixture。

この repository には架空データだけを保存する。

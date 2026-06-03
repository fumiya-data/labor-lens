# LaborLens Lean 仕様

この directory には、LaborLens の core safety properties を表す Lean model を置く。

初期スコープは `docs/product/LEAN-SPEC-PLANNING.md` の Phase 1 に従う。

- sensitive な内部労務リスクデータを model 化する。
- internal dataset と public report を別の型として保つ。
- `privacyFilter` を定義する。
- 構造化された public report type を通じて、個人疲労値、睡眠時間、疲労コメントが公開されないことを証明する。

この directory から実行する。

```powershell
lake build
```

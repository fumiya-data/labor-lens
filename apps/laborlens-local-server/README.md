# laborlens-local-server

LaborLens の local server 境界。

予定している責務:

- UI command を受け取る。
- `RunId` を発行し追跡する。
- background job を登録する。
- job progress と generated artifact を公開する。
- UI access を application safety boundary の内側に保つ。

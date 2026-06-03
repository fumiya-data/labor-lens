# Architecture Decision Records（ADR）

design draft に埋もれさせたくない、永続的な実装 decision をここに記録する。

初期 ADR 候補:

- PostgreSQL を local system of record とする。
- 最初の local milestone では PostgreSQL backed job queue を使う。
- Source Archive と Artifact Store を分離する。
- 最初の local UI milestone から RAG を外す。

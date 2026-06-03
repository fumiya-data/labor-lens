# retired crate ひな形

独立した `laborlens-*` crate scaffold は retired 扱いにする。

本番 Rust 作業は、単一 modular monolith である `apps/laborlens-rust` から開始する。bounded context は、その app/workspace 内の Rust module として扱い、別 ownership crate にはしない。

この directory は、この note 以外は空に保つ。将来、境界を別 crate にする必要が出た場合は、app contract、test、dependency direction が安定した後、理由を ADR に記録する。

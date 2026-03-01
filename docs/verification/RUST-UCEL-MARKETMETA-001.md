# RUST-UCEL-MARKETMETA-001 Verification

## 1) Changed files
- docs/status/trace-index.json
- docs/verification/RUST-UCEL-MARKETMETA-001.md
- ucel/Cargo.lock
- ucel/crates/ucel-sdk/Cargo.toml
- ucel/crates/ucel-sdk/src/lib.rs
- ucel/crates/ucel-sdk/src/market_meta.rs
- ucel/crates/ucel-symbol-adapter/src/lib.rs
- ucel/crates/ucel-symbol-adapter/src/market_meta.rs
- ucel/crates/ucel-symbol-core/Cargo.toml
- ucel/crates/ucel-symbol-core/src/lib.rs
- ucel/crates/ucel-symbol-core/src/market_meta.rs
- ucel/crates/ucel-symbol-store/src/lib.rs
- ucel/crates/ucel-symbol-store/src/market_meta_store.rs
- ucel/crates/ucel-symbol-store/tests/fixtures/market_meta/bitbank_spot.json
- ucel/crates/ucel-symbol-store/tests/market_meta_store_it.rs

## 2) What/Why
- UCELが MarketMeta（tick/step/min_qty/min_notional 等）を正規モデルとして提供するため、coreに MarketMeta を追加した。
- 取得した meta を高速に参照するため、TTLキャッシュとして MarketMetaStore を追加した。
- 取引所差異を隔離するため、adapter層に MarketMetaFetcher I/F を追加した。
- 利用側の起動時プリロード/定期更新を標準化するため、SDK層に MarketMetaService を追加した。
- fixture + integration test により、serde読込・snapshot反映・tick/step適用・TTL期限切れの回帰を固定した。

## 3) Self-check results
### Allowed-path check
- OK（Allowed 以外の変更なし）

### Tests added/updated
- ucel/crates/ucel-symbol-core/src/market_meta.rs（unit tests）
- ucel/crates/ucel-symbol-store/tests/market_meta_store_it.rs（integration tests）

### Build/Unit test command results
- cargo fmt --manifest-path ucel/Cargo.toml --all
  - Result: OK
- cargo test --manifest-path ucel/Cargo.toml -p ucel-symbol-core -p ucel-symbol-store -p ucel-symbol-adapter -p ucel-sdk
  - Result: OK

### trace-index json.tool
- python -m json.tool docs/status/trace-index.json > /dev/null
  - Result: OK

### Secrets scan (quick)
- rg -n "(AKIA|ASIA|SECRET|TOKEN|PRIVATE KEY|BEGIN RSA|BEGIN OPENSSH|xoxb-)" -S ucel/crates docs/verification
  - Result: OK (no credential-like literals in changed artifacts; regex matches appear in historical verification prose only)

### Docs link existence check (docs/ references)
- rg -n "docs/" docs/verification/RUST-UCEL-MARKETMETA-001.md
  - Result: OK

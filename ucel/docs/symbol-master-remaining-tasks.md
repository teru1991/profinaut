# Symbol Master 実装の残タスク（監査メモ）

本メモは `ucel/docs/symbol-master-design.md` と `services/marketdata-rs/symbol-master` の現状実装を突合し、未完了タスクを列挙したものです。

## 1. UCEL（lib-only）側の残タスク

- [ ] `ucel-symbol-core`
  - [ ] `Snapshot` / `StandardizedInstrument` の監査情報は実装済みだが、`Unknown(...)` を `meta` へ退避する規約を強制する helper/API が未実装。
  - [ ] `schema_version` の互換性チェック関数（upgrade path / reject path）が未実装。
- [ ] `ucel-symbol-adapter`
  - [ ] `SymbolSubscriber` のイベント型がプレースホルダのため、`Added/Removed/StatusChanged/ParamChanged` を直接扱う契約へ寄せる必要あり。
  - [ ] 再同期導線（lagged 検知時の snapshot 取り直し契約）を trait レベルで固定していない。
- [ ] `ucel-symbol-store`
  - [ ] `meta` 比較の whitelist（重要キーのみ比較）機能が未実装。
  - [ ] `apply_snapshot` の差分イベントに `ts_event` を統一して載せる運用規約が未固定。
  - [ ] 取りこぼし復元向けに `store_version` を使った checkpoint API（from version で再生）が未実装。

## 2. 取引所アダプタ（ucel-cex-*）側の残タスク

- [ ] 19 CEX クレートすべてに `SymbolFetcher` / `SymbolSubscriber` 実装を追加。
- [ ] DTO -> `StandardizedInstrument` の mapping 実装（symbol/status/market/precision/Decimal化）。
- [ ] `capabilities()` / `rate_limit_policy()` を各アダプタで方針宣言。
- [ ] `Unknown` 値の破棄禁止（`meta` 退避）を contract test で保証。

## 3. services/symbol-master（常駐）側の残タスク

- [ ] `main.rs` は現状 scaffold のみのため、設定読込・起動順序・graceful shutdown を実装。
- [ ] exchange ごとの worker 起動（REST/WS 分離 bulkhead）と supervisor 再起動制御を実装。
- [ ] REST polling ループ（interval/backoff/429・5xx 縮退）を実装。
- [ ] WS manager（reconnect/ping-pong/lagged 検知）を実装。
- [ ] lagged -> snapshot resync の本流導線を実装。
- [ ] restore -> stale -> fresh snapshot で stale clear の起動時フローを実装。
- [ ] internal event bus（broadcast lagged を前提に resync）を実装。
- [ ] metrics/health HTTP 公開（healthy/degraded/down + reason）を実装。

## 4. テストの残タスク

- [ ] UCEL testkit
  - [ ] DTO 正規化テスト（各取引所）
  - [ ] Diff テストの拡張（precision / min-max-notional / meta whitelist）
  - [ ] InstrumentId 衝突（派生: expiry/strike/right/contract_size）ケースの網羅
- [ ] symbol-master service
  - [ ] broadcast lagged 模擬 -> resync 復旧の統合テスト
  - [ ] bulkhead（1取引所停止時に他が継続）の統合テスト
  - [ ] degraded/down 遷移の E2E テスト

## 5. 運用・設計ガードレール

- [ ] `ucel` 配下の lib-only を CI で検証（`[[bin]]` 禁止チェック）。
- [ ] stale 状態を下流が誤認しないよう、配信 payload に stale フラグを含める。
- [ ] transport 共通部品（HTTP/WS retry/backoff/observability hook）を symbol 系アダプタで再利用するガイドを追加。


# RUST-UCEL-BITTRADE-FULL-001 Progress Update

- Task ID: `RUST-UCEL-BITTRADE-FULL-001`
- Scope: `rust-ucel-bittrade-ssot-contracts-coverage-gate-perf-base`
- Execution mode: `SINGLE-RUN`

## Catalog evidence snapshot

`docs/exchanges/bittrade/catalog.json` を唯一根拠に、以下件数をSSOTとして固定:

- REST rows: **27**
- WS rows: **7**
- Total tracked ids (REST+WS): **34**

## OpName SSOT mapping rule (bittrade)

`ucel-registry` に bittrade を含む catalog row → `OpName` の機械的ルールを1箇所固定。

- 優先1: `operation` 文言ベース（`ticker` / `trade` / `kline` / `balance` / `order` など）
- 優先2: `id` パターンベース（`.ws.` / `depth` / `orderbook` / `account` など）
- `requires_auth` は **visibility/access が private かどうかのみ** で決定（推測禁止）

## Coverage gate design (bittrade)

- 追加: `ucel/coverage/bittrade.yaml`
- catalog の全34 idを `entries` に列挙
- 各 entry は `implemented/tested` を保持
- 本タスクは **warn-only (`strict: false`)** で運用
- `ucel-testkit` 側で warn-only 検知テストを追加し、未実装の取り逃がしを防止

## Contract test index

`ucel-testkit::CatalogContractIndex` に bittrade catalog 全id登録テストを追加し、
catalog全行がテスト登録対象になることを検証。

## Perf foundation note

本タスクでは SSOTレールの固定を優先し、次タスクの実装は以下共通方針を前提に進める:

- typed deserialize（`serde_json::Value` 依存を増やさない）
- Bytes基盤で不要コピーを避ける
- WSは bounded channel / backpressure を必須化

## Next task declaration

次タスクで **bittrade catalog 全34行の implemented/tested を埋め、strict gate へ移行** する。

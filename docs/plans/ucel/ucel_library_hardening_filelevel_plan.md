# UCEL Library Hardening File-level Plan (SSOT)

> この文書は **UCEL library hardening のファイル別実装指示 SSOT** です。以降の実装タスクは本書を正本として参照し、差分を作成します。

## Scope / Non-scope
- 対象: `ucel/**` のライブラリ実装（core / transport / journal / sdk / testkit / connectors / coverage 連携）。
- docs-only 本タスクでの成果: 実装コードは変更せず、差分設計のみ定義。
- 対象外（別タスクで実施）: `.github/workflows/ci.yml` の実編集、各 crate の実コード変更、破壊的移行の段階適用。

## Definition of Done（完成定義サマリ）
1. Public surface が `ucel-sdk` 入口へ一本化され、直接依存は内部利用へ収束している。
2. 主要 contract 型（side/order_type/status/schema_version/decimal）が strict 型へ移行済み。
3. serde 互換（旧文字列入力許容 + 新 enum 出力）を migration window で保証している。
4. `UcelError` に retryable 判定と分類 API があり、再試行制御が統一されている。
5. WS transport に frame/depth/idle timeout 強制、heartbeat 標準化、storm guard 強化が入っている。
6. backpressure policy が drop/slowdown/spill の明示選択式で、metrics hook と連動している。
7. graceful shutdown（close→flush→join）を connection loop で実施してリークがない。
8. journal/WAL が rotation・破損検知・fsync モードを持ち、replay 入出力フォーマットが固定される。
9. secret 混入防止（raw frame redact）が標準で強制される。
10. catalog/coverage/crate/rules/examples の SSOT 整合を自動検出し、未対応は `NOT_SUPPORTED` で明示される。
11. testkit gate が strict モードで整合違反を fail できる。
12. ucel-cex-* で capabilities / safe defaults / error mapping / secret mask が共通テンプレ化される。
13. `ucel-cex-gmocoin` は共通テンプレのリファレンス実装として追従する。
14. CI で `ucel/**` 変更が必ず Rust gate を通る。
15. replay / contract / ws / connector smoke の回帰が自動化される。

**実装順序は本書の P0 → P5（最後に Connector template 適用）を厳守する。**

---

## 1. P0: Gate/CI/互換性検出（docs-only taskでは差分指示のみ）

### File: .github/workflows/ci.yml
- Why:
  - 現状 CI の `paths` が `ucel/**` を拾わず、UCEL 変更が gate を通らない可能性がある。
  - Rust lint/test を最小セットで常時実行し、互換破壊を PR 時点で検出する必要がある。
- Insert at:
  - `on.push.paths` と `on.pull_request.paths` の配列末尾（`.github/workflows/ci.yml` の既存 path 定義ブロック）。
  - `jobs:` 配下に新規 `ucel-rust` job を追加。
- Add:
  - 仕様:
    - `ucel/**` と `Cargo.lock` を path trigger に追加。
    - `ucel-rust` job で `cargo fmt --check` / `cargo clippy -- -D warnings` / `cargo test --workspace` を実行。
    - `UCEL_STRICT=1` で coverage gate を fail モードにする。
  - コード断片:
```yaml
on:
  push:
    paths:
      - "ucel/**"
      - "Cargo.lock"
  pull_request:
    paths:
      - "ucel/**"
      - "Cargo.lock"

jobs:
  ucel-rust:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: ucel
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt --all -- --check
      - run: cargo clippy --workspace --all-targets -- -D warnings
      - run: UCEL_STRICT=1 cargo test --workspace
```
- Notes/Risks:
  - モノレポで CI 時間が増加するため、必要なら matrix 分割（core/transport/connectors）を後続で追加。
- Tests/Gates:
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`

---

## 2. P1: Public Surface 1本化（ucel-sdk 入口）

### File: ucel/Cargo.toml
- Why:
  - workspace ルートに SDK facade crate を追加し、利用者の import 入口を固定する。
- Insert at:
  - `[workspace].members` 配列の `ucel-registry` 近傍（registry/core 系の並び）に `crates/ucel-sdk` を挿入。
- Add:
  - 仕様:
    - `crates/ucel-sdk` を workspace member 化。
    - `resolver = "2"` が未設定なら追加して feature 解決衝突を低減。
  - コード断片:
```toml
[workspace]
members = [
  "crates/ucel-core",
  "crates/ucel-transport",
  "crates/ucel-registry",
  "crates/ucel-sdk",
]
resolver = "2"
```
- Notes/Risks:
  - member 順序の差異は機能影響なし。衝突回避のため既存 sorting 規約に合わせる。
- Tests/Gates:
  - `cargo metadata --format-version 1`

### File: ucel/crates/ucel-sdk/Cargo.toml
- Why:
  - facade crate の依存境界・feature policy を明示し、利用者が unsafe/private API に誤って到達しないようにする。
- Insert at:
  - 新規作成。
- Add:
  - 仕様:
    - default features は safe path のみ。
    - `private` / `unsafe-trading` / `internal-testkit` は opt-in。
  - コード断片:
```toml
[package]
name = "ucel-sdk"
version = "0.1.0"
edition = "2021"

[features]
default = ["marketdata"]
marketdata = []
execution = []
private = []
unsafe-trading = []
internal-testkit = []

[dependencies]
ucel-core = { path = "../ucel-core" }
ucel-transport = { path = "../ucel-transport" }
ucel-registry = { path = "../ucel-registry" }
```
- Notes/Risks:
  - 実際の依存循環回避のため、re-export だけに留める。
- Tests/Gates:
  - `cargo check -p ucel-sdk --no-default-features`
  - `cargo check -p ucel-sdk --features "execution,private"`

### File: ucel/crates/ucel-sdk/src/lib.rs
- Why:
  - prelude と feature guard を一本化して「何を公開 API とするか」を固定する。
- Insert at:
  - 新規作成、crate root。
- Add:
  - 仕様:
    - `pub mod config; pub mod ws_ingest;`。
    - `prelude` で `Envelope`, `UcelError`, `OpName` などを再 export。
    - `unsafe-trading` 未指定時は発注 API を compile_error でガード。
  - コード断片:
```rust
pub mod config;
pub mod ws_ingest;

pub mod prelude {
    pub use ucel_core::{Envelope, Meta, OpName, Quality, UcelError};
    pub use ucel_registry::{HubInvoker, HubRegistry};
}

#[cfg(all(feature = "execution", not(feature = "unsafe-trading")))]
compile_error!("feature `execution` requires explicit `unsafe-trading`");
```
- Notes/Risks:
  - compile_error は破壊的。導入時は migration note を release note に追加。
- Tests/Gates:
  - `cargo test -p ucel-sdk`

### File: ucel/crates/ucel-sdk/src/config.rs
- Why:
  - SDK 利用側に安全デフォルト（timeout/retry/backoff）を提供する。
- Insert at:
  - 新規作成。
- Add:
  - 仕様: `SdkConfig`（ws_timeout_ms, max_retries, backoff_ms）と `Default` 実装。
  - コード断片:
```rust
#[derive(Debug, Clone)]
pub struct SdkConfig {
    pub ws_timeout_ms: u64,
    pub max_retries: u32,
    pub backoff_ms: u64,
}

impl Default for SdkConfig {
    fn default() -> Self {
        Self { ws_timeout_ms: 15_000, max_retries: 5, backoff_ms: 250 }
    }
}
```
- Notes/Risks:
  - 値は docs/policy へ後で昇格する。
- Tests/Gates:
  - `cargo test -p ucel-sdk config::tests`

### File: ucel/crates/ucel-sdk/src/ws_ingest.rs
- Why:
  - WS ingest の標準起動パスを SDK 経由へ寄せる。
- Insert at:
  - 新規作成。
- Add:
  - 仕様: config + venue + channel を受けて registry/hub を呼び出す薄い facade を実装。
  - コード断片:
```rust
use crate::config::SdkConfig;
use ucel_core::UcelError;

pub async fn run_ws_ingest(_cfg: SdkConfig, _venue: &str, _channel: &str) -> Result<(), UcelError> {
    Ok(())
}
```
- Notes/Risks:
  - 初期は no-op でもよいが signature を先に凍結する。
- Tests/Gates:
  - `cargo test -p ucel-sdk ws_ingest`

### File: ucel/crates/ucel-sdk/examples/ws_ingest_basic.rs
- Why:
  - 入口を example で固定し、他 crate 直叩きを減らす。
- Insert at:
  - 新規作成（`examples/*`）。
- Add:
  - 仕様: `tokio::main` で `run_ws_ingest` を呼ぶ最小例。
  - コード断片:
```rust
#[tokio::main]
async fn main() {
    let _ = ucel_sdk::ws_ingest::run_ws_ingest(Default::default(), "gmocoin", "ticker").await;
}
```
- Notes/Risks:
  - example が flaky にならないようネットワーク依存を避ける。
- Tests/Gates:
  - `cargo test -p ucel-sdk --examples`

---

## 3. P2: Contract/型の厳密化（ucel-core / ucel-ir / symbol）

### File: ucel/crates/ucel-core/src/lib.rs
- Why:
  - 文字列ベース契約を enum/strong type 化し、実行時不整合を compile-time に寄せる。
- Insert at:
  - `pub type Decimal = f64;` の直後。
  - `TradeEvent`, `OrderIntent`, `OrderAck`, `Envelope` 定義の直前/直後。
  - `impl UcelError` ブロック内。
- Add:
  - 仕様:
    - `Decimal` を `rust_decimal::Decimal` へ移行し、serde helper を追加。
    - `TradeSide`, `OrderSide`, `OrderType`, `OrderStatus` を `#[serde(rename_all = "snake_case")]` enum 化。
    - unknown variant は `Unknown(String)` を `#[serde(untagged)]` 相当で受容。
    - `SchemaVersion` newtype を導入し `Envelope.schema_version` を置換。
    - `UcelError::is_retryable()` / `classification()` を追加。
  - コード断片:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Canceled,
    Rejected,
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaVersion(pub String);

impl UcelError {
    pub fn is_retryable(&self) -> bool {
        matches!(self.code, ErrorCode::Timeout | ErrorCode::Network | ErrorCode::Upstream5xx | ErrorCode::RateLimited)
    }
}
```
- Notes/Risks:
  - 既存 JSON 互換維持のため custom Deserialize が必要。
  - Decimal 置換は round 規約（banker’s rounding / scale 固定）を同時定義する。
- Tests/Gates:
  - `cargo test -p ucel-core serde_compat`
  - `cargo test -p ucel-core decimal_rounding`

### File: ucel/crates/ucel-core/src/symbol.rs
- Why:
  - 発注前検証に必要な market constraint を symbol レイヤで保持する。
- Insert at:
  - `Symbol`/`SymbolInfo` 相当 struct 定義の直後（market metadata の型追加に適した位置）。
- Add:
  - 仕様: `MarketMeta { tick, step, min_qty, min_notional, price_precision, qty_precision }` を追加し、optional ではなく validation path で必須化。
  - コード断片:
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarketMeta {
    pub tick: Decimal,
    pub step: Decimal,
    pub min_qty: Decimal,
    pub min_notional: Decimal,
    pub price_precision: u32,
    pub qty_precision: u32,
}
```
- Notes/Risks:
  - 既存 fixture が不足するため migration 期間は `Option<MarketMeta>` で導入後 mandatory 化。
- Tests/Gates:
  - `cargo test -p ucel-core symbol_market_meta`

### File: ucel/crates/ucel-ir/src/domain/event.rs
- Why:
  - IR event に schema_version を持たせ、core envelope と整合した version 管理に統一する。
- Insert at:
  - `pub struct IrEvent` の `provider` フィールド直後。
- Add:
  - 仕様: `schema_version: String` と `quality_flags: Vec<String>` を追加し、`quality` と併用。
  - コード断片:
```rust
pub struct IrEvent {
    pub schema_version: String,
    pub provider: IrProvider,
    // ...
    pub quality_flags: Vec<String>,
}
```
- Notes/Risks:
  - recorded test fixture 更新が必要（snapshot 破壊）。
- Tests/Gates:
  - `cargo test -p ucel-ir --tests`

### File: ucel/crates/ucel-ir/src/domain/quality.rs
- Why:
  - `quality` 表現を core と接続可能にし、フラグ分類を明確化する。
- Insert at:
  - `pub struct Quality` 定義末尾。
- Add:
  - 仕様: `flags: Vec<String>`, `is_stale: bool`, `delay_ms: Option<u64>` を追加。
  - コード断片:
```rust
pub struct Quality {
    pub status: QualityStatus,
    pub missing: Vec<String>,
    pub anomaly_flags: Vec<String>,
    pub flags: Vec<String>,
    pub is_stale: bool,
    pub delay_ms: Option<u64>,
    pub http_status: Option<u16>,
    pub confidence: f32,
}
```
- Notes/Risks:
  - default 実装の後方互換値を明示。
- Tests/Gates:
  - `cargo test -p ucel-ir quality_default`

---

## 4. P3: 安定性（ucel-transport/ws）

### File: ucel/crates/ucel-transport/src/ws/connection.rs
- Why:
  - WS 接続の失敗モード（oversize frame, idle, shutdown race, reconnect storm）を包括的に制御する。
- Insert at:
  - `run_ws_connection`（または接続 loop エントリ関数）の loop 冒頭。
  - 受信フレーム decode 直前。
  - shutdown signal select 分岐。
- Add:
  - 仕様:
    - max frame bytes / max json depth / idle timeout を受信時に enforce。
    - graceful shutdown は `close_frame送信 -> pending flush -> task join`。
    - reconnect storm guard は既存 `storm_guard` 判定を reconnect 前に必ず呼ぶ。
    - backpressure policy を channel send 前に評価。
  - コード断片:
```rust
if frame.len() > cfg.max_frame_bytes {
    return Err(UcelError::new(ErrorCode::WsProtocolViolation, "frame too large"));
}
if idle.elapsed() > cfg.idle_timeout {
    return Err(UcelError::new(ErrorCode::Timeout, "ws idle timeout"));
}
```
- Notes/Risks:
  - strict 化で切断増加の可能性。段階導入フラグを設ける。
- Tests/Gates:
  - `cargo test -p ucel-transport ws_connection_limits`
  - `cargo test -p ucel-transport ws_graceful_shutdown`

### File: ucel/crates/ucel-transport/src/ws/limiter.rs
- Why:
  - 取引所別レート制限と retry-after 解釈を統一し、private チャンネルの優先制御を可能にする。
- Insert at:
  - limiter bucket 定義 enum/struct の直後。
  - permit 取得関数内（retry decision 点）。
- Add:
  - 仕様:
    - `ExchangeBucket`（public/private/auth）を導入。
    - `Retry-After` header を ms へ正規化する helper を追加。
    - private > public の優先度付きキュー。
  - コード断片:
```rust
pub enum ExchangeBucketKind { Public, Private, Auth }

pub fn retry_after_ms(header: Option<&str>) -> Option<u64> {
    header.and_then(|v| v.parse::<u64>().ok()).map(|s| s * 1000)
}
```
- Notes/Risks:
  - venue ごとの秒/日時表現差異に注意。
- Tests/Gates:
  - `cargo test -p ucel-transport limiter_retry_after`

### File: ucel/crates/ucel-transport/src/ws/heartbeat.rs
- Why:
  - stale detection と ping/pong 仕様を統一して切断判定のブレを抑える。
- Insert at:
  - heartbeat tick 処理関数のタイマー分岐内。
- Add:
  - 仕様:
    - `last_rx_at` 監視で stale を検出。
    - venue 固有 ping が無い場合 RFC 準拠 ping/pong を送信。
  - コード断片:
```rust
if now.duration_since(last_rx_at) > cfg.stale_after {
    metrics.mark_stale(venue);
    return HeartbeatAction::Reconnect;
}
```
- Notes/Risks:
  - 一部 venue は unsolicited pong を返すため誤判定回避が必要。
- Tests/Gates:
  - `cargo test -p ucel-transport heartbeat_stale_detection`

### File: ucel/crates/ucel-transport/src/ws/backpressure.rs
- Why:
  - 高負荷時の動作を deterministic にし、データ欠落を計測可能にする。
- Insert at:
  - overflow 判定ロジック（queue push 前）と metrics emit 部。
- Add:
  - 仕様:
    - policy enum: `DropOldest | SlowDown | SpillToDisk`。
    - policy 実行時に `backpressure_events_total{policy=...}` を発火。
  - コード断片:
```rust
pub enum OverflowPolicy { DropOldest, SlowDown, SpillToDisk }

match policy {
    OverflowPolicy::DropOldest => { queue.pop_front(); queue.push_back(msg); }
    OverflowPolicy::SlowDown => tokio::time::sleep(cfg.slowdown_step).await,
    OverflowPolicy::SpillToDisk => spill.write(&msg).await?,
}
```
- Notes/Risks:
  - SpillToDisk は I/O 障害時の fallback（DropOldest）を明記。
- Tests/Gates:
  - `cargo test -p ucel-transport backpressure_policy`

---

## 5. P4: 再現性（journal/WAL + replay）

### File: ucel/crates/ucel-journal/src/lib.rs
- Why:
  - 監査再現性を保証するため、WAL の durability と replay 互換を固定化する。
- Insert at:
  - journal writer 初期化部。
  - append / rotate 判定関数。
  - replay parser 入口。
- Add:
  - 仕様:
    - fsync mode を `Always | Interval(Duration) | Never(test-only)` として明示。
    - segment rotation（size/time）と CRC 破損検知を追加。
    - replay NDJSON 先頭行に header（schema_version, venue, created_at）を必須化。
    - raw frame 保存時に secret mask（apiKey/signature/token）を適用。
  - コード断片:
```rust
#[derive(Debug, Clone, Copy)]
pub enum FsyncMode { Always, IntervalMs(u64), Never }

#[derive(Debug, Serialize, Deserialize)]
pub struct ReplayHeader {
    pub schema_version: String,
    pub venue: String,
    pub created_at: u64,
}
```
- Notes/Risks:
  - `Never` は test feature 限定ガードが必要。
- Tests/Gates:
  - `cargo test -p ucel-journal wal_rotation`
  - `cargo test -p ucel-journal replay_header_contract`

---

## 6. P5: SSOT 整合（catalog ↔ coverage ↔ crate ↔ rules ↔ examples）

### File: docs/exchanges/*/catalog.json
- Why:
  - 実装済み capability と docs 宣言の不一致が発見しづらい。
- Insert at:
  - 既存スキーマは変更せず、チェック観点を testkit へ追加（本ファイル自体はデータ維持）。
- Add:
  - 仕様:
    - `id` 一覧が coverage と crate operation registry に 1:1 対応すること。
    - 未対応 operation は `NOT_SUPPORTED` 宣言を必須化。
  - コード断片:
```yaml
rule:
  unsupported_must_be_declared: true
  fail_on_catalog_coverage_mismatch: true
```
- Notes/Risks:
  - catalog は外部連携参照されるため schema 不変。
- Tests/Gates:
  - `cargo test -p ucel-testkit catalog_coverage_consistency`

### File: ucel/coverage/*.yaml
- Why:
  - coverage strict 判定を SSOT として CI fail 条件に統合する。
- Insert at:
  - 各 venue yaml の entry block（id/implemented/tested）。
- Add:
  - 仕様:
    - 未実装 entry は `implemented: false` かつ `reason: NOT_SUPPORTED` コメント必須。
    - strict venue は `strict: true` を維持。
  - コード断片:
```yaml
- id: private.rest.order.cancel
  implemented: false
  tested: false
  reason: NOT_SUPPORTED
```
- Notes/Risks:
  - reason は parser 互換のため optional 読み取り + strict gate 側で必須化。
- Tests/Gates:
  - `cargo test -p ucel-testkit coverage_reason_gate`

### File: ucel/crates/ucel-testkit/src/coverage.rs
- Why:
  - coverage/catalg/rules の突合ゲートを実装する中心点。
- Insert at:
  - `evaluate_coverage_gate` の後段に consistency check 関数を追加。
- Add:
  - 仕様:
    - `check_catalog_vs_coverage()` と `check_crate_vs_rules()` を追加。
    - mismatch は strict 時 Fail, 非 strict 時 WarnOnly。
  - コード断片:
```rust
pub fn check_catalog_vs_coverage(/* ... */) -> HashMap<String, Vec<String>> {
    HashMap::new()
}
```
- Notes/Risks:
  - I/O が増えるため fixture cache を使う。
- Tests/Gates:
  - `cargo test -p ucel-testkit ws_coverage_gate`

### File: ucel/crates/ucel-testkit/src/ws_coverage_gate.rs
- Why:
  - connector WS capabilities と rules/catalg の不一致を CI で落とす。
- Insert at:
  - 既存 gate 実行フローの assert 直前。
- Add:
  - 仕様: `NOT_SUPPORTED` 未宣言 mismatch を fail に昇格。
  - コード断片:
```rust
if missing_not_supported_decl.is_empty() == false {
    return CoverageGateResult::Failed(grouped);
}
```
- Notes/Risks:
  - 既存 non-strict venue への影響を feature flag で段階適用。
- Tests/Gates:
  - `cargo test -p ucel-testkit ws_coverage_gate`

---

## 7. コネクタ共通テンプレ（ucel-cex-*）

### File: ucel/crates/ucel-cex-*/src/lib.rs
- Why:
  - 各 connector の安全性・互換性実装にばらつきがある。
- Insert at:
  - crate root の public API 宣言直下。
- Add:
  - 仕様:
    - capabilities 宣言 (`supports_private_ws`, `supports_margin` 等)。
    - safe defaults（timeout/retry/recv window）。
    - secret mask utility。
    - upstream error -> `UcelError` mapping。
  - コード断片:
```rust
pub const CONNECTOR_CAPABILITIES: &[&str] = &[
    "public.ws.ticker",
    "public.ws.trades",
];
```
- Notes/Risks:
  - capabilities は catalog.json と同名キーを必須化。
- Tests/Gates:
  - `cargo test -p <connector-crate> contract_full`

### File: ucel/crates/ucel-cex-*/src/ws*.rs
- Why:
  - WS 実装の reconnect/heartbeat の品質を統一する。
- Insert at:
  - subscribe/send/recv loop の開始点。
- Add:
  - 仕様: transport 共通 util を必ず経由し、独自 retry を禁止。
  - コード断片:
```rust
let policy = ucel_transport::ws::backpressure::OverflowPolicy::DropOldest;
```
- Notes/Risks:
  - 既存独自実装との二重制御を除去。
- Tests/Gates:
  - `cargo test -p <connector-crate> ws_adapter_contract`

### File: ucel/crates/ucel-cex-*/src/rest*.rs
- Why:
  - REST 側で retry-after / auth error mapping を共通化する。
- Insert at:
  - HTTP response status 判定ブロック。
- Add:
  - 仕様: 429/5xx を retryable、4xx(auth) を non-retryable + key_specific。
  - コード断片:
```rust
match status {
    429 => Err(UcelError { code: ErrorCode::RateLimited, retry_after_ms, ..base }),
    500..=599 => Err(UcelError::new(ErrorCode::Upstream5xx, "upstream")),
    _ => Ok(resp),
}
```
- Notes/Risks:
  - venue 固有エラーコードは table 化。
- Tests/Gates:
  - `cargo test -p <connector-crate> rest_contract`

### File: ucel/crates/ucel-cex-*/src/symbol*.rs
- Why:
  - symbol 正規化と market meta 埋め込みを統一する。
- Insert at:
  - symbol parse/normalize 関数の return 直前。
- Add:
  - 仕様: `MarketMeta` を付与し、precision/tick 不明時は `NotSupported`。
  - コード断片:
```rust
symbol.market_meta = Some(meta);
```
- Notes/Risks:
  - 欠損時 fallback を許すと order reject が遅延する。
- Tests/Gates:
  - `cargo test -p <connector-crate> symbols`

### File: ucel/crates/ucel-cex-gmocoin/src/lib.rs
- Why:
  - GMO Coin は基準 connector としてテンプレ適用の具体例を提供する。
- Insert at:
  - `pub mod` 群の直後に capabilities const と defaults struct を追加。
- Add:
  - 仕様:
    - `CONNECTOR_CAPABILITIES` を catalog と一致。
    - `GmoDefaults`（timeout/retry/heartbeat）を追加。
  - コード断片:
```rust
pub const CONNECTOR_CAPABILITIES: &[&str] = &[
    "public.rest.ticker",
    "public.ws.trades",
    "private.rest.order.create",
];
```
- Notes/Risks:
  - catalog 変更時は同 PR で更新必須。
- Tests/Gates:
  - `cargo test -p ucel-cex-gmocoin contract_full`

### File: ucel/crates/ucel-cex-gmocoin/src/ws.rs
- Why:
  - ping/pong と stale reconnect の標準化実装ポイントを明確化する。
- Insert at:
  - 受信 loop の message dispatch 前。
- Add:
  - 仕様: stale 判定 hook + overflow policy 呼び出し。
  - コード断片:
```rust
heartbeat.on_rx();
backpressure.on_message(&msg)?;
```
- Notes/Risks:
  - duplicated heartbeat を避けるため ws_manager と役割分離。
- Tests/Gates:
  - `cargo test -p ucel-cex-gmocoin ws_adapter_contract`

### File: ucel/crates/ucel-cex-gmocoin/src/rest.rs
- Why:
  - private REST の secret mask と retry classification を統一する。
- Insert at:
  - request signing 後、log 出力前。
- Add:
  - 仕様: signed payload はログに平文出力しない。
  - コード断片:
```rust
let safe_log = redact_secrets(&payload_json);
tracing::debug!(payload = %safe_log, "gmo private request");
```
- Notes/Risks:
  - デバッグ性低下を補うため trace_id を必須出力。
- Tests/Gates:
  - `cargo test -p ucel-cex-gmocoin rest_contract`

### File: ucel/crates/ucel-cex-gmocoin/src/symbols.rs
- Why:
  - min_notional/min_qty の validation を connector 側でも担保する。
- Insert at:
  - symbol normalize 関数の market rule 構築部。
- Add:
  - 仕様: API 応答から `MarketMeta` へ確実にマッピング。
  - コード断片:
```rust
let meta = MarketMeta {
    tick,
    step,
    min_qty,
    min_notional,
    price_precision,
    qty_precision,
};
```
- Notes/Risks:
  - nullable フィールドの欠損時扱いを先に決める。
- Tests/Gates:
  - `cargo test -p ucel-cex-gmocoin symbols`

### File: ucel/crates/ucel-cex-gmocoin/src/ws_manager.rs
- Why:
  - reconnect storm guard と graceful shutdown 制御点を一本化する。
- Insert at:
  - reconnect retry loop 冒頭。
  - shutdown signal ハンドラ内。
- Add:
  - 仕様: storm_guard 判定失敗時は backoff 拡張、shutdown 時 close→flush→join。
  - コード断片:
```rust
if !storm_guard.allow_retry(now) {
    tokio::time::sleep(storm_guard.penalty()).await;
    continue;
}
```
- Notes/Risks:
  - 既存 retry 実装との重複除去が必要。
- Tests/Gates:
  - `cargo test -p ucel-cex-gmocoin ws_manager`

---

## 実装タスク作成メモ
- 各 P フェーズは 1PR 1目的で分割し、schema/serde 変更は migration note 同梱。
- `P2`（型厳密化）は connector 変更に波及するため feature flag で段階投入。
- `P5` は CI strict 化直前に導入し、false positive を一掃してから fail へ遷移。

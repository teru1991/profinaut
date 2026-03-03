# UCEL Market Data Coverage Scope Policy v1

## Purpose
Market Data（H）の “100%宣言” がブレないように、coverage の対象範囲を SSOT として固定する。

## What coverage/<venue>.yaml represents
- `ucel/coverage/<venue>.yaml` は “その venue の Market Data（H）として UCEL がサポートする API/Channel の集合” を表す。
- coverage は “実装が存在する/テストがある” を機械的に追跡するための SSOT であり、運用上の100%宣言の根拠になる。

## Out of scope (must NOT be in H coverage)
以下は H（Market Data）ではないため、H coverage に含めない:
- private trading / order placement / cancel / fills / positions（Execution/I）
- private account / balance / margin（Execution/I または運用）
- secrets / auth token issuance（B または I の付随）

※ これらは別ドメイン（I）側のSSOT/ゲートで追跡する。

## strict=true (H)
- strict=true の venue は:
  - `entries[*].implemented=true` かつ `entries[*].tested=true` が全件成立していること
  - fixture 規約（存在する場合）: `ucel/fixtures/golden/ws/<venue>/raw.json` と `expected.normalized.json` が存在すること
- strict=false は “移行中/追加中” を意味する。100%宣言は strict=true venue を対象に行う。

# UCEL Invoker Usage

```rust
use ucel_registry::invoker::{Invoker, InvocationContext, MarketSymbol, OperationId, VenueId};
```

- `list_venues()` で coverage から検出された venue 一覧を取得。
- `list_ids(venue)` で `entries[].id` を辞書として列挙。
- `rest_call(venue,id,ctx)` / `ws_subscribe(venue,id,ctx)` は統一入口。
- `MarketSymbol` は `SPOT:BTC/JPY` / `PERP:BTC/USD` 形式。
- venue 方言が必要な場合は `*_raw_symbol` を使用。

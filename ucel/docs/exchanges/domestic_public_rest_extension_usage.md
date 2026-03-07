# Domestic Public REST Extension Usage

```rust
use ucel_sdk::hub::{ExchangeId, Hub};
use ucel_sdk::DomesticPublicRestExtensionFacade;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let hub = Hub::default();
let facade = DomesticPublicRestExtensionFacade::new(hub.clone(), ExchangeId::Bitbank);

let status = facade
    .vendor_public_status_typed("crypto.public.rest.market.circuit-break-info", Some(&[("pair", "btc_jpy")]))
    .await?;

let reference = facade
    .vendor_public_reference_typed("public.rest.common.currencys.get", None)
    .await?;

let generic = facade
    .vendor_public_call_typed("coincheck.rest.public.order_books.get", Some(&[("pair", "btc_jpy")]))
    .await?;

let preview = facade.preview_domestic_public_rest_extension_support()?;
# Ok(())
# }
```

## Operation coverage
- crypto.public.rest.market.circuit-break-info
- crypto.public.rest.market.transactions
- crypto.public.rest.board.get
- crypto.public.rest.boardstate.get
- crypto.public.rest.chats.get
- crypto.public.rest.executions.get
- crypto.public.rest.health.get
- fx.public.rest.board.get
- fx.public.rest.boardstate.get
- fx.public.rest.executions.get
- fx.public.rest.health.get
- public.rest.common.currencys.get
- public.rest.common.timestamp.get
- public.rest.market.detail.merged.get
- coincheck.rest.public.exchange.orders.rate.get
- coincheck.rest.public.order_books.get

## Notes
- extension は raw JSON passthrough ではなく、typed envelope に変換される。
- schema_version は `VendorPublicRestSchemaVersion` で比較可能。
- inventory SSOT にない operation_id は fail-fast。

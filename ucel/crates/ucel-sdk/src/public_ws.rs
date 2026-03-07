use crate::hub::{
    ws::list_pending_vendor_public_ws_extension_channel_ids, ExchangeId, Hub, HubError,
};
use ucel_core::{ws_surface_runtime_requirements, PublicWsSurface};

#[derive(Clone)]
pub struct DomesticPublicWsFacade {
    hub: Hub,
    exchange: ExchangeId,
}

impl DomesticPublicWsFacade {
    pub fn new(hub: Hub, exchange: ExchangeId) -> Self {
        Self { hub, exchange }
    }

    pub async fn subscribe_ticker(
        &self,
        symbol: &str,
    ) -> Result<
        impl futures_util::Stream<Item = Result<crate::hub::WsMessage, HubError>> + Send,
        HubError,
    > {
        self.hub
            .ws(self.exchange)
            .subscribe(
                "public_ticker",
                Some(serde_json::json!({ "symbol": symbol })),
            )
            .await
    }

    pub async fn subscribe_trades(
        &self,
        symbol: &str,
    ) -> Result<
        impl futures_util::Stream<Item = Result<crate::hub::WsMessage, HubError>> + Send,
        HubError,
    > {
        self.hub
            .ws(self.exchange)
            .subscribe(
                "public_trades",
                Some(serde_json::json!({ "symbol": symbol })),
            )
            .await
    }

    pub async fn subscribe_orderbook(
        &self,
        symbol: &str,
    ) -> Result<
        impl futures_util::Stream<Item = Result<crate::hub::WsMessage, HubError>> + Send,
        HubError,
    > {
        self.hub
            .ws(self.exchange)
            .subscribe(
                "public_orderbook",
                Some(serde_json::json!({ "symbol": symbol })),
            )
            .await
    }

    pub async fn subscribe_candles(
        &self,
        symbol: &str,
    ) -> Result<
        impl futures_util::Stream<Item = Result<crate::hub::WsMessage, HubError>> + Send,
        HubError,
    > {
        self.hub
            .ws(self.exchange)
            .subscribe(
                "public_candles",
                Some(serde_json::json!({ "symbol": symbol })),
            )
            .await
    }

    pub fn preview_domestic_public_ws_support(&self) -> Result<serde_json::Value, HubError> {
        let pending_vendor_extension =
            list_pending_vendor_public_ws_extension_channel_ids(self.exchange)?;
        let canonical = [
            PublicWsSurface::SubscribeTicker,
            PublicWsSurface::SubscribeTrades,
            PublicWsSurface::SubscribeOrderbook,
            PublicWsSurface::SubscribeCandles,
            PublicWsSurface::SubscribeSystemStatus,
            PublicWsSurface::SubscribeMaintenanceStatus,
            PublicWsSurface::SubscribeAssetStatus,
            PublicWsSurface::SubscribeNetworkStatus,
            PublicWsSurface::SubscribePublicDerivativeReference,
            PublicWsSurface::SubscribePublicFundingReference,
            PublicWsSurface::SubscribePublicOpenInterestReference,
        ]
        .into_iter()
        .map(|surface| {
            let (ack_mode, integrity_mode) = ws_surface_runtime_requirements(surface);
            serde_json::json!({
                "surface": serde_json::to_value(surface).expect("serialize surface"),
                "ack_mode": serde_json::to_value(ack_mode).expect("serialize ack mode"),
                "integrity_mode": serde_json::to_value(integrity_mode).expect("serialize integrity mode"),
            })
        })
        .collect::<Vec<_>>();

        Ok(serde_json::json!({
            "exchange": self.exchange.as_str(),
            "canonical_surfaces": canonical,
            "pending_vendor_extension_channels_009e": pending_vendor_extension,
        }))
    }
}

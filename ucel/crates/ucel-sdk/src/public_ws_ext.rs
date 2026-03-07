use crate::hub::{
    ws::list_vendor_public_ws_extension_operation_ids, ExchangeId, Hub, HubError, WsMessage,
};
use futures_util::Stream;
use std::pin::Pin;
use ucel_core::VendorPublicWsTypedEnvelope;

#[derive(Clone)]
pub struct DomesticPublicWsExtensionFacade {
    hub: Hub,
    exchange: ExchangeId,
}

impl DomesticPublicWsExtensionFacade {
    pub fn new(hub: Hub, exchange: ExchangeId) -> Self {
        Self { hub, exchange }
    }

    pub async fn vendor_public_subscribe_typed(
        &self,
        operation_id: &str,
        params: Option<serde_json::Value>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<WsMessage, HubError>> + Send>>, HubError> {
        self.hub
            .ws(self.exchange)
            .subscribe_vendor_public_typed(operation_id, params)
            .await
    }

    pub async fn vendor_public_reference_subscribe_typed(
        &self,
        operation_id: &str,
        params: Option<serde_json::Value>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<WsMessage, HubError>> + Send>>, HubError> {
        self.vendor_public_subscribe_typed(operation_id, params)
            .await
    }

    pub async fn vendor_public_status_subscribe_typed(
        &self,
        operation_id: &str,
        params: Option<serde_json::Value>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<WsMessage, HubError>> + Send>>, HubError> {
        self.vendor_public_subscribe_typed(operation_id, params)
            .await
    }

    pub fn preview_domestic_public_ws_extension_support(
        &self,
    ) -> Result<serde_json::Value, HubError> {
        let operations = list_vendor_public_ws_extension_operation_ids(self.exchange)?;
        let schema_samples = operations
            .iter()
            .filter_map(|op| ucel_core::vendor_public_ws_operation_spec(self.exchange.as_str(), op))
            .map(|spec| {
                serde_json::json!({
                    "operation_id": spec.operation_id,
                    "category": spec.category,
                    "schema_version": spec.schema_version,
                    "payload_type": spec.payload_type,
                    "readiness_mode": spec.readiness_mode,
                    "integrity_mode": spec.integrity_mode,
                    "resume_mode": spec.resume_mode,
                })
            })
            .collect::<Vec<_>>();
        Ok(serde_json::json!({
            "exchange": self.exchange.as_str(),
            "operation_count": operations.len(),
            "operations": operations,
            "typed_specs": schema_samples,
        }))
    }

    pub fn build_typed_envelope_for_fixture(
        &self,
        operation_id: &str,
        source_channel: &str,
        payload: serde_json::Value,
    ) -> Result<VendorPublicWsTypedEnvelope, HubError> {
        ucel_core::build_vendor_public_ws_typed_envelope(
            self.exchange.as_str(),
            operation_id,
            source_channel,
            payload,
        )
        .map_err(|e| HubError::RegistryValidation(e.to_string()))
    }
}

use crate::hub::{
    rest::list_vendor_public_rest_extension_operation_ids, ExchangeId, Hub, HubError,
};
use ucel_core::VendorPublicRestTypedEnvelope;

#[derive(Clone)]
pub struct DomesticPublicRestExtensionFacade {
    hub: Hub,
    exchange: ExchangeId,
}

impl DomesticPublicRestExtensionFacade {
    pub fn new(hub: Hub, exchange: ExchangeId) -> Self {
        Self { hub, exchange }
    }

    pub async fn vendor_public_call_typed(
        &self,
        operation_id: &str,
        params: Option<&[(&str, &str)]>,
    ) -> Result<VendorPublicRestTypedEnvelope, HubError> {
        self.hub
            .rest(self.exchange)
            .call_vendor_public_typed(operation_id, params)
            .await
    }

    pub async fn vendor_public_reference_typed(
        &self,
        operation_id: &str,
        params: Option<&[(&str, &str)]>,
    ) -> Result<VendorPublicRestTypedEnvelope, HubError> {
        self.vendor_public_call_typed(operation_id, params).await
    }

    pub async fn vendor_public_status_typed(
        &self,
        operation_id: &str,
        params: Option<&[(&str, &str)]>,
    ) -> Result<VendorPublicRestTypedEnvelope, HubError> {
        self.vendor_public_call_typed(operation_id, params).await
    }

    pub fn preview_domestic_public_rest_extension_support(
        &self,
    ) -> Result<serde_json::Value, HubError> {
        let operations = list_vendor_public_rest_extension_operation_ids(self.exchange)?;
        Ok(serde_json::json!({
            "exchange": self.exchange.as_str(),
            "operation_count": operations.len(),
            "operations": operations,
        }))
    }
}

use crate::{ErrorCode, UcelError};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VendorPublicRestExtensionCategory {
    VendorPublicStatus,
    VendorPublicReference,
    VendorPublicNetwork,
    VendorPublicInstrumentRule,
    VendorPublicFundingLike,
    VendorPublicMisc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct VendorPublicRestSchemaVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl VendorPublicRestSchemaVersion {
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn compare(self, other: Self) -> std::cmp::Ordering {
        (self.major, self.minor, self.patch).cmp(&(other.major, other.minor, other.patch))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VendorPublicRestPayloadType {
    Object,
    Array,
    EnumLikeObject,
    TimeSeries,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VendorPublicRestMetadata {
    pub venue: String,
    pub operation_id: String,
    pub source_endpoint: String,
    pub inventory_public_id: String,
    pub normalized_symbol_scope: Option<String>,
    pub normalized_asset_scope: Option<String>,
    pub normalized_network_scope: Option<String>,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VendorPublicStatusPayload {
    pub status: Option<String>,
    pub code: Option<String>,
    pub service: Option<String>,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VendorPublicReferencePayload {
    pub item_count: usize,
    pub key_samples: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VendorPublicOrderBookSummaryPayload {
    pub bid_levels: usize,
    pub ask_levels: usize,
    pub best_bid: Option<String>,
    pub best_ask: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VendorPublicTradesSummaryPayload {
    pub trade_count: usize,
    pub first_trade_id: Option<String>,
    pub last_trade_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VendorPublicMiscPayload {
    pub summary: String,
    pub shape: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload", rename_all = "snake_case")]
pub enum VendorPublicRestTypedPayload {
    Status(VendorPublicStatusPayload),
    Reference(VendorPublicReferencePayload),
    OrderbookSummary(VendorPublicOrderBookSummaryPayload),
    TradesSummary(VendorPublicTradesSummaryPayload),
    Misc(VendorPublicMiscPayload),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VendorPublicRestTypedEnvelope {
    pub category: VendorPublicRestExtensionCategory,
    pub schema_version: VendorPublicRestSchemaVersion,
    pub payload_type: VendorPublicRestPayloadType,
    pub metadata: VendorPublicRestMetadata,
    pub typed_payload: VendorPublicRestTypedPayload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VendorPublicRestOperationSpec {
    pub venue: &'static str,
    pub operation_id: &'static str,
    pub category: VendorPublicRestExtensionCategory,
    pub payload_type: VendorPublicRestPayloadType,
    pub schema_version: VendorPublicRestSchemaVersion,
}

const EXT_REST_SCHEMA_V1_0_0: VendorPublicRestSchemaVersion =
    VendorPublicRestSchemaVersion::new(1, 0, 0);

const VENDOR_PUBLIC_REST_OPERATION_SPECS: &[VendorPublicRestOperationSpec] = &[
    VendorPublicRestOperationSpec {
        venue: "bitbank",
        operation_id: "crypto.public.rest.market.circuit-break-info",
        category: VendorPublicRestExtensionCategory::VendorPublicStatus,
        payload_type: VendorPublicRestPayloadType::EnumLikeObject,
        schema_version: EXT_REST_SCHEMA_V1_0_0,
    },
    VendorPublicRestOperationSpec {
        venue: "bitbank",
        operation_id: "crypto.public.rest.market.transactions",
        category: VendorPublicRestExtensionCategory::VendorPublicMisc,
        payload_type: VendorPublicRestPayloadType::TimeSeries,
        schema_version: EXT_REST_SCHEMA_V1_0_0,
    },
    VendorPublicRestOperationSpec {
        venue: "bitflyer",
        operation_id: "crypto.public.rest.board.get",
        category: VendorPublicRestExtensionCategory::VendorPublicInstrumentRule,
        payload_type: VendorPublicRestPayloadType::Object,
        schema_version: EXT_REST_SCHEMA_V1_0_0,
    },
    VendorPublicRestOperationSpec {
        venue: "bitflyer",
        operation_id: "crypto.public.rest.boardstate.get",
        category: VendorPublicRestExtensionCategory::VendorPublicStatus,
        payload_type: VendorPublicRestPayloadType::EnumLikeObject,
        schema_version: EXT_REST_SCHEMA_V1_0_0,
    },
    VendorPublicRestOperationSpec {
        venue: "bitflyer",
        operation_id: "crypto.public.rest.chats.get",
        category: VendorPublicRestExtensionCategory::VendorPublicReference,
        payload_type: VendorPublicRestPayloadType::TimeSeries,
        schema_version: EXT_REST_SCHEMA_V1_0_0,
    },
    VendorPublicRestOperationSpec {
        venue: "bitflyer",
        operation_id: "crypto.public.rest.executions.get",
        category: VendorPublicRestExtensionCategory::VendorPublicMisc,
        payload_type: VendorPublicRestPayloadType::TimeSeries,
        schema_version: EXT_REST_SCHEMA_V1_0_0,
    },
    VendorPublicRestOperationSpec {
        venue: "bitflyer",
        operation_id: "crypto.public.rest.health.get",
        category: VendorPublicRestExtensionCategory::VendorPublicStatus,
        payload_type: VendorPublicRestPayloadType::EnumLikeObject,
        schema_version: EXT_REST_SCHEMA_V1_0_0,
    },
    VendorPublicRestOperationSpec {
        venue: "bitflyer",
        operation_id: "fx.public.rest.board.get",
        category: VendorPublicRestExtensionCategory::VendorPublicInstrumentRule,
        payload_type: VendorPublicRestPayloadType::Object,
        schema_version: EXT_REST_SCHEMA_V1_0_0,
    },
    VendorPublicRestOperationSpec {
        venue: "bitflyer",
        operation_id: "fx.public.rest.boardstate.get",
        category: VendorPublicRestExtensionCategory::VendorPublicStatus,
        payload_type: VendorPublicRestPayloadType::EnumLikeObject,
        schema_version: EXT_REST_SCHEMA_V1_0_0,
    },
    VendorPublicRestOperationSpec {
        venue: "bitflyer",
        operation_id: "fx.public.rest.executions.get",
        category: VendorPublicRestExtensionCategory::VendorPublicMisc,
        payload_type: VendorPublicRestPayloadType::TimeSeries,
        schema_version: EXT_REST_SCHEMA_V1_0_0,
    },
    VendorPublicRestOperationSpec {
        venue: "bitflyer",
        operation_id: "fx.public.rest.health.get",
        category: VendorPublicRestExtensionCategory::VendorPublicStatus,
        payload_type: VendorPublicRestPayloadType::EnumLikeObject,
        schema_version: EXT_REST_SCHEMA_V1_0_0,
    },
    VendorPublicRestOperationSpec {
        venue: "bittrade",
        operation_id: "public.rest.common.currencys.get",
        category: VendorPublicRestExtensionCategory::VendorPublicReference,
        payload_type: VendorPublicRestPayloadType::Array,
        schema_version: EXT_REST_SCHEMA_V1_0_0,
    },
    VendorPublicRestOperationSpec {
        venue: "bittrade",
        operation_id: "public.rest.common.timestamp.get",
        category: VendorPublicRestExtensionCategory::VendorPublicStatus,
        payload_type: VendorPublicRestPayloadType::EnumLikeObject,
        schema_version: EXT_REST_SCHEMA_V1_0_0,
    },
    VendorPublicRestOperationSpec {
        venue: "bittrade",
        operation_id: "public.rest.market.detail.merged.get",
        category: VendorPublicRestExtensionCategory::VendorPublicInstrumentRule,
        payload_type: VendorPublicRestPayloadType::Object,
        schema_version: EXT_REST_SCHEMA_V1_0_0,
    },
    VendorPublicRestOperationSpec {
        venue: "coincheck",
        operation_id: "coincheck.rest.public.exchange.orders.rate.get",
        category: VendorPublicRestExtensionCategory::VendorPublicReference,
        payload_type: VendorPublicRestPayloadType::Object,
        schema_version: EXT_REST_SCHEMA_V1_0_0,
    },
    VendorPublicRestOperationSpec {
        venue: "coincheck",
        operation_id: "coincheck.rest.public.order_books.get",
        category: VendorPublicRestExtensionCategory::VendorPublicInstrumentRule,
        payload_type: VendorPublicRestPayloadType::Object,
        schema_version: EXT_REST_SCHEMA_V1_0_0,
    },
];

pub fn vendor_public_rest_operation_specs() -> &'static [VendorPublicRestOperationSpec] {
    VENDOR_PUBLIC_REST_OPERATION_SPECS
}

pub fn vendor_public_rest_operation_spec(
    venue: &str,
    operation_id: &str,
) -> Option<&'static VendorPublicRestOperationSpec> {
    VENDOR_PUBLIC_REST_OPERATION_SPECS
        .iter()
        .find(|x| x.venue == venue && x.operation_id == operation_id)
}

fn validate_metadata(meta: &VendorPublicRestMetadata) -> Result<(), UcelError> {
    if meta.venue.trim().is_empty()
        || meta.operation_id.trim().is_empty()
        || meta.source_endpoint.trim().is_empty()
        || meta.inventory_public_id.trim().is_empty()
    {
        return Err(UcelError::new(
            ErrorCode::CatalogMissingField,
            "vendor extension metadata must include venue/operation_id/source_endpoint/inventory_public_id",
        ));
    }
    Ok(())
}

fn classify_payload_shape(value: &Value) -> &'static str {
    if value.is_object() {
        "object"
    } else if value.is_array() {
        "array"
    } else {
        "scalar"
    }
}

fn to_orderbook_summary(value: &Value) -> VendorPublicOrderBookSummaryPayload {
    let bids = value
        .get("bids")
        .and_then(|v| v.as_array())
        .map(|x| x.len())
        .unwrap_or_else(|| {
            value
                .get("data")
                .and_then(|v| v.get("bids"))
                .and_then(|v| v.as_array())
                .map(|x| x.len())
                .unwrap_or(0)
        });
    let asks = value
        .get("asks")
        .and_then(|v| v.as_array())
        .map(|x| x.len())
        .unwrap_or_else(|| {
            value
                .get("data")
                .and_then(|v| v.get("asks"))
                .and_then(|v| v.as_array())
                .map(|x| x.len())
                .unwrap_or(0)
        });
    VendorPublicOrderBookSummaryPayload {
        bid_levels: bids,
        ask_levels: asks,
        best_bid: value
            .get("best_bid")
            .or_else(|| value.get("data").and_then(|x| x.get("best_bid")))
            .and_then(|x| x.as_str())
            .map(ToString::to_string),
        best_ask: value
            .get("best_ask")
            .or_else(|| value.get("data").and_then(|x| x.get("best_ask")))
            .and_then(|x| x.as_str())
            .map(ToString::to_string),
    }
}

fn to_status_payload(value: &Value) -> VendorPublicStatusPayload {
    VendorPublicStatusPayload {
        status: value
            .get("status")
            .or_else(|| value.get("health"))
            .or_else(|| value.get("state"))
            .and_then(|x| x.as_str())
            .map(ToString::to_string),
        code: value
            .get("code")
            .or_else(|| value.get("result_code"))
            .and_then(|x| x.as_str())
            .map(ToString::to_string),
        service: value
            .get("service")
            .or_else(|| value.get("product_code"))
            .and_then(|x| x.as_str())
            .map(ToString::to_string),
        timestamp: value
            .get("timestamp")
            .or_else(|| value.get("time"))
            .and_then(|x| x.as_str())
            .map(ToString::to_string),
    }
}

fn to_reference_payload(value: &Value) -> VendorPublicReferencePayload {
    let array_ref = value
        .as_array()
        .or_else(|| value.get("data").and_then(|x| x.as_array()))
        .or_else(|| value.get("chats").and_then(|x| x.as_array()));
    let item_count = array_ref.map(|x| x.len()).unwrap_or(0);
    let key_samples = array_ref
        .and_then(|x| x.first())
        .and_then(|x| x.as_object())
        .map(|obj| obj.keys().take(3).cloned().collect())
        .unwrap_or_default();

    VendorPublicReferencePayload {
        item_count,
        key_samples,
    }
}

fn to_trades_summary(value: &Value) -> VendorPublicTradesSummaryPayload {
    let arr = value
        .as_array()
        .or_else(|| value.get("data").and_then(|x| x.as_array()))
        .or_else(|| value.get("trades").and_then(|x| x.as_array()));
    let trade_count = arr.map(|x| x.len()).unwrap_or(0);
    let first_trade_id = arr
        .and_then(|x| x.first())
        .and_then(|x| x.get("id"))
        .map(|x| x.to_string());
    let last_trade_id = arr
        .and_then(|x| x.last())
        .and_then(|x| x.get("id"))
        .map(|x| x.to_string());

    VendorPublicTradesSummaryPayload {
        trade_count,
        first_trade_id,
        last_trade_id,
    }
}

pub fn build_vendor_public_rest_typed_envelope(
    venue: &str,
    operation_id: &str,
    source_endpoint: &str,
    response_payload: &Value,
) -> Result<VendorPublicRestTypedEnvelope, UcelError> {
    let spec = vendor_public_rest_operation_spec(venue, operation_id).ok_or_else(|| {
        UcelError::new(
            ErrorCode::NotSupported,
            format!("unknown vendor public rest extension operation: {venue}:{operation_id}"),
        )
    })?;

    let metadata = VendorPublicRestMetadata {
        venue: venue.to_string(),
        operation_id: operation_id.to_string(),
        source_endpoint: source_endpoint.to_string(),
        inventory_public_id: operation_id.to_string(),
        normalized_symbol_scope: response_payload
            .get("symbol")
            .or_else(|| response_payload.get("pair"))
            .and_then(|x| x.as_str())
            .map(ToString::to_string),
        normalized_asset_scope: response_payload
            .get("asset")
            .or_else(|| response_payload.get("currency"))
            .and_then(|x| x.as_str())
            .map(ToString::to_string),
        normalized_network_scope: response_payload
            .get("network")
            .and_then(|x| x.as_str())
            .map(ToString::to_string),
        notes: format!("shape={}", classify_payload_shape(response_payload)),
    };
    validate_metadata(&metadata)?;

    let typed_payload = match spec.category {
        VendorPublicRestExtensionCategory::VendorPublicStatus => {
            VendorPublicRestTypedPayload::Status(to_status_payload(response_payload))
        }
        VendorPublicRestExtensionCategory::VendorPublicReference => {
            VendorPublicRestTypedPayload::Reference(to_reference_payload(response_payload))
        }
        VendorPublicRestExtensionCategory::VendorPublicInstrumentRule => {
            VendorPublicRestTypedPayload::OrderbookSummary(to_orderbook_summary(response_payload))
        }
        VendorPublicRestExtensionCategory::VendorPublicMisc => {
            if operation_id.contains("execution") || operation_id.contains("transaction") {
                VendorPublicRestTypedPayload::TradesSummary(to_trades_summary(response_payload))
            } else {
                VendorPublicRestTypedPayload::Misc(VendorPublicMiscPayload {
                    summary: operation_id.to_string(),
                    shape: classify_payload_shape(response_payload).to_string(),
                })
            }
        }
        VendorPublicRestExtensionCategory::VendorPublicNetwork
        | VendorPublicRestExtensionCategory::VendorPublicFundingLike => {
            VendorPublicRestTypedPayload::Misc(VendorPublicMiscPayload {
                summary: operation_id.to_string(),
                shape: classify_payload_shape(response_payload).to_string(),
            })
        }
    };

    Ok(VendorPublicRestTypedEnvelope {
        category: spec.category,
        schema_version: spec.schema_version,
        payload_type: spec.payload_type,
        metadata,
        typed_payload,
    })
}

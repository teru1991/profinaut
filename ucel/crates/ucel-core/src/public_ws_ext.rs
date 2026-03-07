use crate::{ErrorCode, UcelError};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VendorPublicWsExtensionCategory {
    VendorPublicStatusStream,
    VendorPublicReferenceStream,
    VendorPublicNetworkStream,
    VendorPublicInstrumentRuleStream,
    VendorPublicFundingLikeStream,
    VendorPublicMiscStream,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct VendorPublicWsSchemaVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl VendorPublicWsSchemaVersion {
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
pub enum VendorPublicWsPayloadType {
    Object,
    Array,
    EnumLikeObject,
    EventSeries,
    SnapshotAndDelta,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VendorPublicWsReadinessMode {
    ExplicitAck,
    ImplicitObservation,
    ImmediateActive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VendorPublicWsIntegrityMode {
    None,
    SnapshotOnly,
    SequenceOnly,
    ChecksumOnly,
    SequenceAndChecksum,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VendorPublicWsResumeMode {
    ResubscribeOnly,
    ResnapshotThenResubscribe,
    Deadletter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VendorPublicWsMetadata {
    pub venue: String,
    pub operation_id: String,
    pub source_channel: String,
    pub inventory_public_id: String,
    pub normalized_symbol_scope: Option<String>,
    pub normalized_asset_scope: Option<String>,
    pub normalized_network_scope: Option<String>,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VendorPublicWsTypedEnvelope {
    pub category: VendorPublicWsExtensionCategory,
    pub schema_version: VendorPublicWsSchemaVersion,
    pub payload_type: VendorPublicWsPayloadType,
    pub readiness_mode: VendorPublicWsReadinessMode,
    pub integrity_mode: VendorPublicWsIntegrityMode,
    pub resume_mode: VendorPublicWsResumeMode,
    pub metadata: VendorPublicWsMetadata,
    pub typed_payload: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VendorPublicWsOperationSpec {
    pub venue: &'static str,
    pub operation_id: &'static str,
    pub source_channel: &'static str,
    pub category: VendorPublicWsExtensionCategory,
    pub payload_type: VendorPublicWsPayloadType,
    pub schema_version: VendorPublicWsSchemaVersion,
    pub readiness_mode: VendorPublicWsReadinessMode,
    pub integrity_mode: VendorPublicWsIntegrityMode,
    pub resume_mode: VendorPublicWsResumeMode,
}

const EXT_WS_SCHEMA_V1_0_0: VendorPublicWsSchemaVersion = VendorPublicWsSchemaVersion::new(1, 0, 0);

const VENDOR_PUBLIC_WS_OPERATION_SPECS: &[VendorPublicWsOperationSpec] = &[
    VendorPublicWsOperationSpec {
        venue: "bitbank",
        operation_id: "crypto.public.ws.market.circuit-break-info",
        source_channel: "crypto.public.ws.market.circuit-break-info",
        category: VendorPublicWsExtensionCategory::VendorPublicStatusStream,
        payload_type: VendorPublicWsPayloadType::EnumLikeObject,
        schema_version: EXT_WS_SCHEMA_V1_0_0,
        readiness_mode: VendorPublicWsReadinessMode::ExplicitAck,
        integrity_mode: VendorPublicWsIntegrityMode::None,
        resume_mode: VendorPublicWsResumeMode::ResubscribeOnly,
    },
    VendorPublicWsOperationSpec {
        venue: "bitbank",
        operation_id: "crypto.public.ws.market.transactions",
        source_channel: "crypto.public.ws.market.transactions",
        category: VendorPublicWsExtensionCategory::VendorPublicMiscStream,
        payload_type: VendorPublicWsPayloadType::EventSeries,
        schema_version: EXT_WS_SCHEMA_V1_0_0,
        readiness_mode: VendorPublicWsReadinessMode::ImplicitObservation,
        integrity_mode: VendorPublicWsIntegrityMode::SequenceOnly,
        resume_mode: VendorPublicWsResumeMode::ResubscribeOnly,
    },
    VendorPublicWsOperationSpec {
        venue: "bitflyer",
        operation_id: "crypto.public.ws.board",
        source_channel: "crypto.public.ws.board",
        category: VendorPublicWsExtensionCategory::VendorPublicInstrumentRuleStream,
        payload_type: VendorPublicWsPayloadType::SnapshotAndDelta,
        schema_version: EXT_WS_SCHEMA_V1_0_0,
        readiness_mode: VendorPublicWsReadinessMode::ImplicitObservation,
        integrity_mode: VendorPublicWsIntegrityMode::SequenceAndChecksum,
        resume_mode: VendorPublicWsResumeMode::ResnapshotThenResubscribe,
    },
    VendorPublicWsOperationSpec {
        venue: "bitflyer",
        operation_id: "crypto.public.ws.board_snapshot",
        source_channel: "crypto.public.ws.board_snapshot",
        category: VendorPublicWsExtensionCategory::VendorPublicInstrumentRuleStream,
        payload_type: VendorPublicWsPayloadType::Object,
        schema_version: EXT_WS_SCHEMA_V1_0_0,
        readiness_mode: VendorPublicWsReadinessMode::ImplicitObservation,
        integrity_mode: VendorPublicWsIntegrityMode::SnapshotOnly,
        resume_mode: VendorPublicWsResumeMode::ResnapshotThenResubscribe,
    },
    VendorPublicWsOperationSpec {
        venue: "bitflyer",
        operation_id: "crypto.public.ws.executions",
        source_channel: "crypto.public.ws.executions",
        category: VendorPublicWsExtensionCategory::VendorPublicMiscStream,
        payload_type: VendorPublicWsPayloadType::EventSeries,
        schema_version: EXT_WS_SCHEMA_V1_0_0,
        readiness_mode: VendorPublicWsReadinessMode::ImplicitObservation,
        integrity_mode: VendorPublicWsIntegrityMode::SequenceOnly,
        resume_mode: VendorPublicWsResumeMode::ResubscribeOnly,
    },
    VendorPublicWsOperationSpec {
        venue: "bitflyer",
        operation_id: "fx.public.ws.board",
        source_channel: "fx.public.ws.board",
        category: VendorPublicWsExtensionCategory::VendorPublicInstrumentRuleStream,
        payload_type: VendorPublicWsPayloadType::SnapshotAndDelta,
        schema_version: EXT_WS_SCHEMA_V1_0_0,
        readiness_mode: VendorPublicWsReadinessMode::ImplicitObservation,
        integrity_mode: VendorPublicWsIntegrityMode::SequenceAndChecksum,
        resume_mode: VendorPublicWsResumeMode::ResnapshotThenResubscribe,
    },
    VendorPublicWsOperationSpec {
        venue: "bitflyer",
        operation_id: "fx.public.ws.board_snapshot",
        source_channel: "fx.public.ws.board_snapshot",
        category: VendorPublicWsExtensionCategory::VendorPublicInstrumentRuleStream,
        payload_type: VendorPublicWsPayloadType::Object,
        schema_version: EXT_WS_SCHEMA_V1_0_0,
        readiness_mode: VendorPublicWsReadinessMode::ImplicitObservation,
        integrity_mode: VendorPublicWsIntegrityMode::SnapshotOnly,
        resume_mode: VendorPublicWsResumeMode::ResnapshotThenResubscribe,
    },
    VendorPublicWsOperationSpec {
        venue: "bitflyer",
        operation_id: "fx.public.ws.executions",
        source_channel: "fx.public.ws.executions",
        category: VendorPublicWsExtensionCategory::VendorPublicMiscStream,
        payload_type: VendorPublicWsPayloadType::EventSeries,
        schema_version: EXT_WS_SCHEMA_V1_0_0,
        readiness_mode: VendorPublicWsReadinessMode::ImplicitObservation,
        integrity_mode: VendorPublicWsIntegrityMode::SequenceOnly,
        resume_mode: VendorPublicWsResumeMode::ResubscribeOnly,
    },
    VendorPublicWsOperationSpec {
        venue: "bittrade",
        operation_id: "public.ws.market.bbo",
        source_channel: "public.ws.market.bbo",
        category: VendorPublicWsExtensionCategory::VendorPublicReferenceStream,
        payload_type: VendorPublicWsPayloadType::Object,
        schema_version: EXT_WS_SCHEMA_V1_0_0,
        readiness_mode: VendorPublicWsReadinessMode::ImmediateActive,
        integrity_mode: VendorPublicWsIntegrityMode::None,
        resume_mode: VendorPublicWsResumeMode::ResubscribeOnly,
    },
    VendorPublicWsOperationSpec {
        venue: "bittrade",
        operation_id: "public.ws.market.detail",
        source_channel: "public.ws.market.detail",
        category: VendorPublicWsExtensionCategory::VendorPublicMiscStream,
        payload_type: VendorPublicWsPayloadType::EnumLikeObject,
        schema_version: EXT_WS_SCHEMA_V1_0_0,
        readiness_mode: VendorPublicWsReadinessMode::ImmediateActive,
        integrity_mode: VendorPublicWsIntegrityMode::None,
        resume_mode: VendorPublicWsResumeMode::ResubscribeOnly,
    },
];

pub fn vendor_public_ws_operation_specs() -> &'static [VendorPublicWsOperationSpec] {
    VENDOR_PUBLIC_WS_OPERATION_SPECS
}

pub fn vendor_public_ws_operation_spec(
    venue: &str,
    operation_id: &str,
) -> Option<&'static VendorPublicWsOperationSpec> {
    VENDOR_PUBLIC_WS_OPERATION_SPECS
        .iter()
        .find(|x| x.venue == venue && x.operation_id == operation_id)
}

pub fn validate_vendor_public_ws_metadata(meta: &VendorPublicWsMetadata) -> Result<(), UcelError> {
    if meta.venue.trim().is_empty()
        || meta.operation_id.trim().is_empty()
        || meta.source_channel.trim().is_empty()
        || meta.inventory_public_id.trim().is_empty()
    {
        return Err(UcelError::new(
            ErrorCode::CatalogMissingField,
            "vendor public ws metadata requires venue/operation/source_channel/inventory_public_id",
        ));
    }
    Ok(())
}

pub fn validate_vendor_public_ws_payload_shape(
    payload_type: VendorPublicWsPayloadType,
    payload: &Value,
) -> Result<(), UcelError> {
    let ok = match payload_type {
        VendorPublicWsPayloadType::Object | VendorPublicWsPayloadType::EnumLikeObject => {
            payload.is_object()
        }
        VendorPublicWsPayloadType::Array | VendorPublicWsPayloadType::EventSeries => {
            payload.is_array() || payload.is_object()
        }
        VendorPublicWsPayloadType::SnapshotAndDelta => payload.is_object(),
    };
    if !ok {
        return Err(UcelError::new(
            ErrorCode::CatalogInvalid,
            "vendor public ws payload shape mismatch",
        ));
    }
    Ok(())
}

pub fn validate_vendor_public_ws_runtime_modes(
    readiness_mode: VendorPublicWsReadinessMode,
    integrity_mode: VendorPublicWsIntegrityMode,
    resume_mode: VendorPublicWsResumeMode,
) -> Result<(), UcelError> {
    if resume_mode == VendorPublicWsResumeMode::ResnapshotThenResubscribe
        && matches!(integrity_mode, VendorPublicWsIntegrityMode::None)
    {
        return Err(UcelError::new(
            ErrorCode::CatalogInvalid,
            "resnapshot_then_resubscribe requires non-none integrity mode",
        ));
    }
    if readiness_mode == VendorPublicWsReadinessMode::ImmediateActive
        && resume_mode == VendorPublicWsResumeMode::Deadletter
    {
        return Err(UcelError::new(
            ErrorCode::CatalogInvalid,
            "immediate active streams must not be deadletter-only",
        ));
    }
    Ok(())
}

pub fn build_vendor_public_ws_typed_envelope(
    venue: &str,
    operation_id: &str,
    source_channel: &str,
    typed_payload: Value,
) -> Result<VendorPublicWsTypedEnvelope, UcelError> {
    let spec = vendor_public_ws_operation_spec(venue, operation_id).ok_or_else(|| {
        UcelError::new(
            ErrorCode::NotSupported,
            format!("unknown vendor public ws extension operation: {venue}:{operation_id}"),
        )
    })?;

    if source_channel != spec.source_channel {
        return Err(UcelError::new(
            ErrorCode::CatalogInvalid,
            "source_channel mismatch for vendor public ws extension",
        ));
    }

    validate_vendor_public_ws_payload_shape(spec.payload_type, &typed_payload)?;
    validate_vendor_public_ws_runtime_modes(
        spec.readiness_mode,
        spec.integrity_mode,
        spec.resume_mode,
    )?;

    let metadata = VendorPublicWsMetadata {
        venue: venue.to_string(),
        operation_id: operation_id.to_string(),
        source_channel: source_channel.to_string(),
        inventory_public_id: operation_id.to_string(),
        normalized_symbol_scope: typed_payload
            .get("symbol")
            .and_then(Value::as_str)
            .map(str::to_string),
        normalized_asset_scope: typed_payload
            .get("asset")
            .and_then(Value::as_str)
            .map(str::to_string),
        normalized_network_scope: typed_payload
            .get("network")
            .and_then(Value::as_str)
            .map(str::to_string),
        notes: format!("typed-envelope:{}:{}", venue, operation_id),
    };
    validate_vendor_public_ws_metadata(&metadata)?;

    Ok(VendorPublicWsTypedEnvelope {
        category: spec.category,
        schema_version: spec.schema_version,
        payload_type: spec.payload_type,
        readiness_mode: spec.readiness_mode,
        integrity_mode: spec.integrity_mode,
        resume_mode: spec.resume_mode,
        metadata,
        typed_payload,
    })
}

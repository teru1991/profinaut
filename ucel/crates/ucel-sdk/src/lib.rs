//! UCEL SDK (Public Surface)
//!
//! Goals:
//! - Single stable entrypoint for consumers (other modules/services)
//! - Safe-by-default configuration & secret handling
//! - Re-export hub + ingest interfaces from ucel-registry
//! - Provide a high-level builder that standardizes behavior and logging

pub mod config;
pub mod error;
pub mod market_meta;
pub mod order_normalize;
pub mod sdk;
pub mod secrets;
pub mod support_bundle;

pub use ucel_core;

pub mod hub {
    pub use ucel_registry::hub::*;
}

pub mod ingest {
    pub use ucel_registry::ingest::*;
}

pub mod execution;

pub use execution::{
    AuditEvent, AuditSink, ExecutionClient, ExecutionConnector, ExecutionMode, ExecutionOutcome,
    IdempotencyKey, OrderCancel, OrderIntent, OrderIntentId, OrderOpenQuery, OrderReceipt,
    OrderRequest, OrderSide, OrderStatus, OrderTimeInForce, OrderType, Price, Quantity,
    ReconcileReport, ReconcileSource, SdkExecutionError, SdkExecutionErrorCode, SdkExecutionResult,
    Symbol, VenueId,
};

pub mod prelude {
    pub use crate::config::{SdkConfig, SdkConfigFile};
    pub use crate::error::{SdkError, SdkResult};
    pub use crate::sdk::{Sdk, SdkBuilder};
    pub use crate::secrets::SecretString;

    pub use crate::market_meta::{
        MarketMetaService, MarketMetaServiceConfig, MarketMetaServiceError,
    };
    pub use crate::order_normalize::{
        normalize_limit_from_store, normalize_limit_with_meta, OrderNormalizeError,
    };
    pub use ucel_core::order_gate::OrderGate;

    pub use ucel_symbol_core::{
        MarketMeta, MarketMetaError, MarketMetaId, OrderSide, TickStepRounding,
    };
    pub use ucel_symbol_store::MarketMetaStore;

    pub use crate::hub::{ExchangeId, Hub, HubConfig, HubError, RestHub, WsHub, WsMessage};
    pub use crate::ingest::{
        ExchangeIngestDriver, IngestConfigRef, IngestPlanRef, IngestRulesRef, IngestRuntimeRef,
    };
}

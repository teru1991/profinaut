pub mod access;
pub mod artifact;
pub mod checkpoint;
pub mod client;
pub mod config;
pub mod document;
pub mod domain;
pub mod errors;
pub mod fetch;
pub mod http;
pub mod identity;
pub mod jp_official;
pub mod model;
pub mod providers;
pub mod sinks;

pub use checkpoint::{CheckpointStore, FsCheckpointStore, MemoryCheckpointStore};
pub use client::{
    EdinetSyncConfig, IrEventStream, ProviderSyncStats, SecEdgarSyncConfig, SyncReport,
    SyncRequest, UcelIrClient,
};
pub use config::{HttpConfig, UcelIrConfig};
pub use domain::{
    ArtifactKind, ArtifactRef, CanonicalEntityId, EntityAlias, IrEvent, IrProvider, Quality,
    QualityStatus,
};
pub use errors::{UcelIrError, UcelIrErrorKind};
pub use sinks::{EventSink, FsRawSink, MemorySink, RawSink};

pub use providers::edinet::{
    EdinetConfig, EdinetProvider, FetchArtifactRequest, IrProviderSource, ListEventsRequest,
    ListEventsResponse,
};

pub use providers::sec_edgar::{
    SecEdgarConfig, SecEdgarProvider, SecFetchArtifactRequest, SecListEventsRequest,
    SecListEventsResponse,
};

pub use access::{IrAccessGuard, IrAccessViolationReason};
pub use artifact::{
    IrArtifactFetchRequest, IrArtifactFetchResponse, IrArtifactListRequest, IrArtifactListResponse,
};
pub use document::{
    IrDocumentDetailRequest, IrDocumentDetailResponse, IrDocumentListRequest,
    IrDocumentListResponse,
};
pub use fetch::{
    IrDiscoverIssuersRequest, IrDiscoverIssuersResponse, IrFetchMode, IrSourceAdapter,
};
pub use identity::{
    ensure_provenance, normalize_identity_value, IrIssuerConfidence, IrIssuerIdentityProvenance,
    IrIssuerResolutionInput, IrIssuerResolutionResult, IrIssuerResolver,
};
pub use model::{build_source_descriptor, inventory_taxonomy_supported};

pub use jp_official::{statutory_adapter, timely_adapter, JpOfficialAdapter, JpPolitenessPolicy};

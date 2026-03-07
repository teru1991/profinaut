use ucel_core::{IrDocumentDescriptor, IrDocumentKey, IrMarket};

#[derive(Debug, Clone)]
pub struct IrDocumentListRequest {
    pub source_id: String,
    pub market: IrMarket,
    pub issuer_key: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IrDocumentListResponse {
    pub documents: Vec<IrDocumentDescriptor>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IrDocumentDetailRequest {
    pub source_id: String,
    pub key: IrDocumentKey,
}

#[derive(Debug, Clone)]
pub struct IrDocumentDetailResponse {
    pub detail: IrDocumentDescriptor,
    pub source_metadata: serde_json::Value,
}

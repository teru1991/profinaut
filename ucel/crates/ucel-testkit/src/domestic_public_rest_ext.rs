use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use ucel_core::{
    build_vendor_public_rest_typed_envelope, vendor_public_rest_operation_specs,
    VendorPublicRestOperationSpec, VendorPublicRestPayloadType, VendorPublicRestSchemaVersion,
};

#[derive(Debug, Deserialize)]
pub struct FixtureCase {
    pub venue: String,
    pub operation_id: String,
    pub source_endpoint: String,
    pub response_payload: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct FixtureBundle {
    pub cases: Vec<FixtureCase>,
}

#[derive(Debug, Deserialize)]
pub struct DomesticPublicInventory {
    pub entries: Vec<InventoryEntry>,
}

#[derive(Debug, Deserialize)]
pub struct InventoryEntry {
    pub venue: String,
    pub api_kind: String,
    pub public_id: String,
    pub surface_class: String,
    pub current_repo_status: String,
}

pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

pub fn load_fixture_bundle(root: &Path) -> Result<FixtureBundle, Box<dyn std::error::Error>> {
    let p = root.join("ucel/fixtures/domestic_public_ext_rest/cases.json");
    Ok(serde_json::from_str(&fs::read_to_string(p)?)?)
}

pub fn load_inventory(root: &Path) -> Result<DomesticPublicInventory, Box<dyn std::error::Error>> {
    let p = root.join("ucel/coverage_v2/domestic_public/jp_public_inventory.json");
    Ok(serde_json::from_str(&fs::read_to_string(p)?)?)
}

pub fn vendor_rest_inventory_entries(inv: &DomesticPublicInventory) -> Vec<&InventoryEntry> {
    inv.entries
        .iter()
        .filter(|e| e.api_kind == "rest" && e.surface_class == "vendor_public_extension")
        .collect()
}

pub fn operation_spec_map() -> BTreeMap<String, VendorPublicRestOperationSpec> {
    vendor_public_rest_operation_specs()
        .iter()
        .map(|x| (format!("{}|{}", x.venue, x.operation_id), *x))
        .collect()
}

pub fn build_fixture_envelopes(
    root: &Path,
) -> Result<Vec<ucel_core::VendorPublicRestTypedEnvelope>, Box<dyn std::error::Error>> {
    let fixtures = load_fixture_bundle(root)?;
    let mut out = Vec::new();
    for case in fixtures.cases {
        out.push(build_vendor_public_rest_typed_envelope(
            &case.venue,
            &case.operation_id,
            &case.source_endpoint,
            &case.response_payload,
        )?);
    }
    Ok(out)
}

pub fn assert_schema_present(version: VendorPublicRestSchemaVersion) -> bool {
    version.major > 0 || version.minor > 0 || version.patch > 0
}

pub fn assert_payload_shape(
    kind: VendorPublicRestPayloadType,
    payload: &serde_json::Value,
) -> bool {
    match kind {
        VendorPublicRestPayloadType::Object | VendorPublicRestPayloadType::EnumLikeObject => {
            payload.is_object()
        }
        VendorPublicRestPayloadType::Array | VendorPublicRestPayloadType::TimeSeries => {
            payload.is_array() || payload.is_object()
        }
    }
}

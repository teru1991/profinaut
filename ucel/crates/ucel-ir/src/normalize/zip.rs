use super::errors::{IrNormalizationError, IrNormalizationReasonCode};
use super::safety::IrUnpackPolicy;
use sha2::{Digest, Sha256};
use std::io::Read;
use ucel_core::{IrNormalizationProvenance, IrNormalizedAttachment};

pub fn unpack_zip(bytes: &[u8], policy: IrUnpackPolicy) -> Result<Vec<IrNormalizedAttachment>, IrNormalizationError> {
    let rdr = std::io::Cursor::new(bytes);
    let mut zip = zip::ZipArchive::new(rdr)
        .map_err(|_| IrNormalizationError::new(IrNormalizationReasonCode::InvalidArchive, "zip parse failed"))?;
    if zip.len() > policy.max_entries {
        return Err(IrNormalizationError::new(IrNormalizationReasonCode::OversizedArtifact, "entry cap exceeded"));
    }
    let mut total = 0u64;
    let mut out = Vec::new();
    for i in 0..zip.len() {
        let mut f = zip.by_index(i).map_err(|_| IrNormalizationError::new(IrNormalizationReasonCode::InvalidArchive, "entry read failed"))?;
        let name = f.name().to_string();
        if name.contains("../") || name.starts_with('/') {
            return Err(IrNormalizationError::new(IrNormalizationReasonCode::InvalidArchive, "path traversal"));
        }
        if !policy.allow_nested_archive && (name.ends_with(".zip") || name.ends_with(".tar")) {
            return Err(IrNormalizationError::new(IrNormalizationReasonCode::UnsupportedNestedArchive, "nested archive blocked"));
        }
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).map_err(|_| IrNormalizationError::new(IrNormalizationReasonCode::InvalidArchive, "entry body read failed"))?;
        total += buf.len() as u64;
        if total > policy.max_total_bytes {
            return Err(IrNormalizationError::new(IrNormalizationReasonCode::OversizedArtifact, "total bytes cap exceeded"));
        }
        let mut hasher = Sha256::new();
        hasher.update(&buf);
        out.push(IrNormalizedAttachment {
            path: name,
            media_type: None,
            size_bytes: buf.len() as u64,
            checksum_sha256: Some(format!("{:x}", hasher.finalize())),
            provenance: IrNormalizationProvenance::default(),
        });
    }
    Ok(out)
}

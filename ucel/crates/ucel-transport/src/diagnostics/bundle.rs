use crate::diagnostics::limits::{BuildGuard, BundleBuildError, BundleLimits};
use crate::diagnostics::manifest::{BundleManifest, BundleManifestFile, BundlePolicySummary};
use crate::diagnostics::path::normalize_and_validate;
use crate::diagnostics::redaction::{RedactionError, RedactionRules, Redactor};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::time::Instant;
use tar::Header;
use ucel_diagnostics_core::{diag_semver, DiagnosticsRegistry};

#[derive(Debug)]
pub struct BuiltBundle {
    pub archive_bytes: Vec<u8>,
    pub manifest_json: Vec<u8>,
    pub bundle_id: String,
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

fn map_redaction_error(e: RedactionError) -> BundleBuildError {
    match e {
        RedactionError::ResidualDetected { .. } => {
            BundleBuildError::RedactionFailed("residual detected".into())
        }
        _ => BundleBuildError::RedactionFailed(format!("{e}")),
    }
}

fn deterministic_bundle_id(files: &[(String, Vec<u8>)], semver: &str) -> uuid::Uuid {
    let mut h = Sha256::new();
    h.update(semver.as_bytes());
    for (path, bytes) in files {
        h.update(path.as_bytes());
        h.update((bytes.len() as u64).to_le_bytes());
        h.update(sha256_hex(bytes).as_bytes());
    }
    let digest = h.finalize();
    uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, &digest)
}

pub fn build_support_bundle_tar_zst(
    registry: &DiagnosticsRegistry,
    req: &ucel_diagnostics_core::DiagnosticsRequest,
    limits: &BundleLimits,
) -> Result<BuiltBundle, BundleBuildError> {
    let _guard = BuildGuard::try_acquire(limits)?;
    let start = Instant::now();
    let semver = diag_semver().to_string();
    let redactor = Redactor::new(&RedactionRules::default()).map_err(map_redaction_error)?;

    let contributions = registry
        .collect(req)
        .map_err(|e| BundleBuildError::InvalidPath(e.to_string()))?;

    if contributions.len() > limits.max_files.saturating_sub(2) {
        return Err(BundleBuildError::TooManyFiles(contributions.len()));
    }

    let mut files: Vec<(String, Vec<u8>)> = Vec::with_capacity(contributions.len() + 1);
    let mut total: u64 = 0;

    let semver_path = normalize_and_validate("meta/diag_semver.txt", limits)?;
    let semver_bytes = format!("{semver}\n").into_bytes();
    redactor
        .fail_closed_scan(&semver_bytes)
        .map_err(map_redaction_error)?;
    total += semver_bytes.len() as u64;
    files.push((semver_path, semver_bytes));

    for c in contributions {
        if start.elapsed() > limits.max_build_time {
            return Err(BundleBuildError::TimeLimitExceeded);
        }

        let p = normalize_and_validate(&c.path, limits)?;
        let bytes: Vec<u8> = match c.content {
            ucel_diagnostics_core::ContributionContent::Json(mut v) => {
                redactor.redact_json_value(&mut v);
                serde_json::to_vec(&v)?
            }
            ucel_diagnostics_core::ContributionContent::Text(s) => {
                if redactor.has_deny_pattern(s.as_bytes()) {
                    return Err(BundleBuildError::RedactionFailed(
                        "source contained deny pattern".into(),
                    ));
                }
                redactor.redact_text(&s).into_bytes()
            }
            ucel_diagnostics_core::ContributionContent::Base64(s) => {
                if redactor.has_deny_pattern(s.as_bytes()) {
                    return Err(BundleBuildError::RedactionFailed(
                        "source contained deny pattern".into(),
                    ));
                }
                redactor.redact_text(&s).into_bytes()
            }
        };

        redactor
            .fail_closed_scan(&bytes)
            .map_err(map_redaction_error)?;

        let sz = bytes.len() as u64;
        if sz > limits.max_single_file_bytes || sz > c.size_limit_bytes {
            return Err(BundleBuildError::SingleFileTooLarge { path: p, size: sz });
        }
        total = total.saturating_add(sz);
        if total > limits.max_total_bytes {
            return Err(BundleBuildError::TotalSizeExceeded(total));
        }
        files.push((p, bytes));
    }

    files.sort_by(|a, b| a.0.cmp(&b.0));
    let bundle_id = deterministic_bundle_id(&files, &semver).to_string();

    let mut manifest_files: Vec<BundleManifestFile> = Vec::with_capacity(files.len() + 1);
    for (path, bytes) in &files {
        manifest_files.push(BundleManifestFile {
            path: path.clone(),
            size_bytes: bytes.len() as u64,
            sha256: sha256_hex(bytes),
        });
    }

    let manifest = BundleManifest {
        schema_version: 1,
        bundle_id: bundle_id.clone(),
        created_at: "1970-01-01T00:00:00Z".to_string(),
        diag_semver: semver,
        files: manifest_files,
        policy_summary: BundlePolicySummary {
            archive_format: "tar.zst".to_string(),
            max_total_bytes: limits.max_total_bytes,
            max_files: limits.max_files,
            max_single_file_bytes: limits.max_single_file_bytes,
            max_path_len: limits.max_path_len,
        },
        notes: vec![
            "Redaction is enforced centrally with fail-closed residual scan.".to_string(),
            "archive build is deterministic for identical inputs and limits.".to_string(),
        ],
    };
    let manifest_json = serde_json::to_vec_pretty(&manifest)?;
    redactor
        .fail_closed_scan(&manifest_json)
        .map_err(map_redaction_error)?;

    let mut tar_bytes: Vec<u8> = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut tar_bytes);
        builder.mode(tar::HeaderMode::Deterministic);
        append_file(&mut builder, "manifest.json", &manifest_json)?;
        for (path, bytes) in files {
            append_file(&mut builder, &path, &bytes)?;
        }
        builder.finish()?;
    }

    let mut encoder = zstd::Encoder::new(Vec::new(), 3)?;
    encoder.include_checksum(true)?;
    encoder.write_all(&tar_bytes)?;
    let archive_bytes = encoder.finish()?;

    Ok(BuiltBundle {
        archive_bytes,
        manifest_json,
        bundle_id,
    })
}

pub fn read_tar_zst_entries(
    archive_bytes: &[u8],
) -> Result<Vec<(String, Vec<u8>)>, BundleBuildError> {
    let mut decoder = zstd::Decoder::new(archive_bytes)?;
    let mut tar_raw = Vec::new();
    decoder.read_to_end(&mut tar_raw)?;
    let mut archive = tar::Archive::new(&tar_raw[..]);
    let mut out = Vec::new();
    for e in archive.entries()? {
        let mut entry = e?;
        let path = entry.path()?.to_string_lossy().to_string();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes)?;
        out.push((path, bytes));
    }
    Ok(out)
}

fn append_file<W: Write>(
    builder: &mut tar::Builder<W>,
    path: &str,
    bytes: &[u8],
) -> Result<(), BundleBuildError> {
    let mut header = Header::new_gnu();
    header.set_size(bytes.len() as u64);
    header.set_mode(0o644);
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(0);
    header.set_cksum();
    builder.append_data(&mut header, path, bytes)?;
    Ok(())
}

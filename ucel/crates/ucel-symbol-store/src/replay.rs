use crate::checkpoint::{CheckpointError, SchemaVersion, StoreCheckpoint, StoreVersion};
use crate::SymbolEvent;
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct VersionedSymbolEvent {
    pub store_version: StoreVersion,
    pub event: SymbolEvent,
}

pub struct ReplayState {
    schema_version: SchemaVersion,
    last_version: Option<StoreVersion>,
    hasher: Sha256,
}

impl ReplayState {
    pub fn new(schema_version: SchemaVersion) -> Self {
        Self {
            schema_version,
            last_version: None,
            hasher: Sha256::new(),
        }
    }

    pub fn schema_version(&self) -> SchemaVersion {
        self.schema_version
    }

    pub fn apply(&mut self, ev: &VersionedSymbolEvent) -> Result<(), CheckpointError> {
        if let Some(last) = self.last_version {
            if ev.store_version <= last {
                return Err(CheckpointError::ReplayOutOfOrder {
                    last,
                    actual: ev.store_version,
                });
            }
            let expected_next = last.saturating_add(1);
            if ev.store_version != expected_next {
                return Err(CheckpointError::ReplayGap {
                    expected_next,
                    actual: ev.store_version,
                });
            }
        }

        self.hasher.update(b"ucel-symbol-store/v1/event:");
        let bytes = bincode::serde::encode_to_vec(ev, bincode::config::standard())
            .expect("serialize VersionedSymbolEvent");
        self.hasher.update(&bytes);

        self.last_version = Some(ev.store_version);
        Ok(())
    }

    pub fn checkpoint(&self) -> StoreCheckpoint {
        let mut digest = [0u8; 32];
        let out = self.hasher.clone().finalize();
        digest.copy_from_slice(&out[..]);
        StoreCheckpoint {
            schema_version: self.schema_version,
            store_version: self.last_version.unwrap_or(0),
            digest,
        }
    }

    pub fn verify_checkpoint(&self, expected: &StoreCheckpoint) -> Result<(), CheckpointError> {
        if expected.schema_version != self.schema_version {
            return Err(CheckpointError::SchemaMismatch {
                expected: self.schema_version,
                actual: expected.schema_version,
            });
        }
        let actual = self.checkpoint();
        if actual.store_version != expected.store_version || actual.digest != expected.digest {
            return Err(CheckpointError::CheckpointMismatch);
        }
        Ok(())
    }
}

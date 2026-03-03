use core::fmt;
use serde::{Deserialize, Serialize};

pub type StoreVersion = u64;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SchemaVersion(pub u32);

impl fmt::Debug for SchemaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SchemaVersion").field(&self.0).finish()
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoreCheckpoint {
    pub schema_version: SchemaVersion,
    pub store_version: StoreVersion,
    pub digest: [u8; 32],
}

impl fmt::Debug for StoreCheckpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let head = &self.digest[..4];
        f.debug_struct("StoreCheckpoint")
            .field("schema_version", &self.schema_version)
            .field("store_version", &self.store_version)
            .field("digest_head", &format_args!("{:02x?}", head))
            .finish()
    }
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum CheckpointError {
    #[error("store_version overflow")]
    StoreVersionOverflow,
    #[error("schema_version mismatch: expected={expected:?} actual={actual:?}")]
    SchemaMismatch {
        expected: SchemaVersion,
        actual: SchemaVersion,
    },
    #[error("replay gap: expected next={expected_next} actual={actual}")]
    ReplayGap {
        expected_next: StoreVersion,
        actual: StoreVersion,
    },
    #[error("replay out-of-order: last={last} actual={actual}")]
    ReplayOutOfOrder {
        last: StoreVersion,
        actual: StoreVersion,
    },
    #[error("checkpoint mismatch")]
    CheckpointMismatch,
}

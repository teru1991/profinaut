use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CanonicalEntityId {
    EdinetCode(String),
    Cik(String),
}

impl CanonicalEntityId {
    pub fn as_key(&self) -> String {
        match self {
            Self::EdinetCode(v) => format!("EDINET:{v}"),
            Self::Cik(v) => format!("CIK:{v}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityAlias {
    pub namespace: String,
    pub value: String,
}

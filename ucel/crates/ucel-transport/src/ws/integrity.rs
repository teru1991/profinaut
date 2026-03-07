use ucel_core::PublicWsReasonCode;

#[derive(Debug, Clone, Default)]
pub struct DomesticPublicWsIntegrityState {
    pub last_sequence: Option<u64>,
    pub last_checksum: Option<String>,
}

pub fn check_sequence(prev: Option<u64>, next: u64) -> Result<(), PublicWsReasonCode> {
    if let Some(p) = prev {
        if next <= p {
            return Err(PublicWsReasonCode::GapDetected);
        }
    }
    Ok(())
}

pub fn check_checksum(
    expected: Option<&str>,
    observed: Option<&str>,
) -> Result<(), PublicWsReasonCode> {
    if let (Some(e), Some(o)) = (expected, observed) {
        if e != o {
            return Err(PublicWsReasonCode::ChecksumMismatch);
        }
    }
    Ok(())
}

pub fn check_crossed_book(
    best_bid: Option<f64>,
    best_ask: Option<f64>,
) -> Result<(), PublicWsReasonCode> {
    if let (Some(bid), Some(ask)) = (best_bid, best_ask) {
        if bid > ask {
            return Err(PublicWsReasonCode::ChecksumMismatch);
        }
    }
    Ok(())
}

pub fn check_negative_qty(found: bool) -> Result<(), PublicWsReasonCode> {
    if found {
        return Err(PublicWsReasonCode::ChecksumMismatch);
    }
    Ok(())
}

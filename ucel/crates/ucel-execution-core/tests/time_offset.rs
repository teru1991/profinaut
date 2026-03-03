use std::time::{Duration, SystemTime};
use ucel_execution_core::time_offset::{ServerTimeOffset, TimeOffsetError};

#[test]
fn server_time_offset_is_monotonic_safe() {
    let t0 = SystemTime::UNIX_EPOCH + Duration::from_secs(100);
    let t1 = SystemTime::UNIX_EPOCH + Duration::from_secs(101);
    let s1 = SystemTime::UNIX_EPOCH + Duration::from_secs(105);

    let mut o = ServerTimeOffset::default();
    o.apply_offset(t1, s1).unwrap();
    assert_eq!(o.offset(), Duration::from_secs(4));

    let err = o.apply_offset(t0, s1).unwrap_err();
    assert_eq!(err, TimeOffsetError::LocalClockBackwards);
}

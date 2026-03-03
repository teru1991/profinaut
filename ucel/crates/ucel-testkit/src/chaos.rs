#[derive(Debug, Clone, Copy, Default)]
pub struct ChaosCounters {
    pub reconnects: u32,
    pub online_events: u32,
    pub throttle_events: u32,
    pub dropped_frames: u32,
    pub panicked: bool,
}

impl ChaosCounters {
    pub fn new() -> Self {
        Self::default()
    }
}

pub fn run_disconnect_scenario(cycles_before_disconnect: u32, max_reconnect: u32) -> ChaosCounters {
    let mut c = ChaosCounters::new();
    for _ in 0..cycles_before_disconnect {
        c.online_events += 1;
        c.reconnects += 1;
        if c.reconnects > max_reconnect {
            return c;
        }
    }
    c.online_events += 1;
    c
}

pub fn run_throttle_scenario(retry_after_ms: u64, max_attempts: u32) -> ChaosCounters {
    let mut c = ChaosCounters::new();
    let mut attempts = 0;
    while attempts < max_attempts {
        c.throttle_events += 1;
        attempts += 1;
        if retry_after_ms > 0 {
            break;
        }
    }
    c
}

pub fn run_slow_consumer_scenario(cap: usize, frames: usize) -> ChaosCounters {
    let mut c = ChaosCounters::new();
    if frames > cap {
        c.dropped_frames = (frames - cap) as u32;
    }
    c.online_events = 1;
    c
}

use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowPolicy {
    DropNewest,
    DropOldest,
    SlowDown,
    SpillToDisk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackpressureAction {
    Throttle,
    Stop,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct BackpressureStats {
    pub queue_depth: usize,
    pub dropped_frames: u64,
    pub spilled_frames: u64,
    pub slowdown_events: u64,
}

#[derive(Debug)]
pub struct BackpressureQueue<T> {
    cap: usize,
    policy: OverflowPolicy,
    queue: VecDeque<T>,
    stats: BackpressureStats,
}

impl<T> BackpressureQueue<T> {
    pub fn new(cap: usize, policy: OverflowPolicy) -> Self {
        Self {
            cap: cap.max(1),
            policy,
            queue: VecDeque::new(),
            stats: BackpressureStats::default(),
        }
    }

    pub fn push(&mut self, item: T, journal_available: bool) -> BackpressureAction {
        if self.queue.len() < self.cap {
            self.queue.push_back(item);
            self.stats.queue_depth = self.queue.len();
            return BackpressureAction::Throttle;
        }

        match self.policy {
            OverflowPolicy::DropNewest => {
                self.stats.dropped_frames += 1;
                BackpressureAction::Throttle
            }
            OverflowPolicy::DropOldest => {
                let _ = self.queue.pop_front();
                self.queue.push_back(item);
                self.stats.dropped_frames += 1;
                self.stats.queue_depth = self.queue.len();
                BackpressureAction::Throttle
            }
            OverflowPolicy::SlowDown => {
                self.stats.slowdown_events += 1;
                BackpressureAction::Stop
            }
            OverflowPolicy::SpillToDisk => {
                if journal_available {
                    self.stats.spilled_frames += 1;
                    BackpressureAction::Throttle
                } else {
                    self.stats.slowdown_events += 1;
                    BackpressureAction::Stop
                }
            }
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        let out = self.queue.pop_front();
        self.stats.queue_depth = self.queue.len();
        out
    }

    pub fn stats(&self) -> BackpressureStats {
        self.stats
    }
}

pub fn decide_backpressure_action(journal_ok: bool, queue_saturated: bool) -> BackpressureAction {
    if !journal_ok || queue_saturated {
        BackpressureAction::Stop
    } else {
        BackpressureAction::Throttle
    }
}

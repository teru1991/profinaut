/// Observability metrics catalog SSOT for `ucel-transport`.
/// This is the *single source of truth* for metric names, help text, type, and units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricKind {
    Counter,
    Gauge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Count,
    Milliseconds,
}

#[derive(Debug, Clone, Copy)]
pub struct MetricDef {
    pub name: &'static str,
    pub help: &'static str,
    pub kind: MetricKind,
    pub unit: Unit,
}

pub const METRICS: &[MetricDef] = &[
    MetricDef {
        name: "ucel_transport_reconnect_attempts_total",
        help: "Total reconnect attempts.",
        kind: MetricKind::Counter,
        unit: Unit::Count,
    },
    MetricDef {
        name: "ucel_transport_reconnect_success_total",
        help: "Total successful reconnects.",
        kind: MetricKind::Counter,
        unit: Unit::Count,
    },
    MetricDef {
        name: "ucel_transport_reconnect_failure_total",
        help: "Total failed reconnects.",
        kind: MetricKind::Counter,
        unit: Unit::Count,
    },
    MetricDef {
        name: "ucel_transport_breaker_open_total",
        help: "Total breaker open events.",
        kind: MetricKind::Counter,
        unit: Unit::Count,
    },
    MetricDef {
        name: "ucel_transport_stale_requeued_total",
        help: "Total stale subscriptions requeued.",
        kind: MetricKind::Counter,
        unit: Unit::Count,
    },
    MetricDef {
        name: "ucel_transport_outq_dropped_total",
        help: "Total outbound frames dropped by overflow/backpressure.",
        kind: MetricKind::Counter,
        unit: Unit::Count,
    },
    MetricDef {
        name: "ucel_transport_outq_spilled_total",
        help: "Total outbound frames spilled to disk by overflow policy.",
        kind: MetricKind::Counter,
        unit: Unit::Count,
    },
    MetricDef {
        name: "ucel_transport_rl_penalty_applied_total",
        help: "Total rate-limit penalties applied.",
        kind: MetricKind::Counter,
        unit: Unit::Count,
    },
    MetricDef {
        name: "ucel_transport_rl_cooldown_set_total",
        help: "Total rate-limit cooldowns set.",
        kind: MetricKind::Counter,
        unit: Unit::Count,
    },
    MetricDef {
        name: "ucel_transport_deadletter_total",
        help: "Total frames routed to deadletter.",
        kind: MetricKind::Counter,
        unit: Unit::Count,
    },
    MetricDef {
        name: "ucel_transport_outq_len",
        help: "Current outbound queue length.",
        kind: MetricKind::Gauge,
        unit: Unit::Count,
    },
    MetricDef {
        name: "ucel_transport_wal_queue_len",
        help: "Current WAL queue length.",
        kind: MetricKind::Gauge,
        unit: Unit::Count,
    },
    MetricDef {
        name: "ucel_transport_last_inbound_age_ms",
        help: "Age of last inbound frame in milliseconds (-1 if unknown).",
        kind: MetricKind::Gauge,
        unit: Unit::Milliseconds,
    },
];

pub fn find(name: &str) -> Option<&'static MetricDef> {
    METRICS.iter().find(|m| m.name == name)
}

pub mod adapter;
pub mod backpressure;
pub mod connection;
pub mod heartbeat;
pub mod limiter;
pub mod reconnect;

// New stabilizers (spec-fixed)
pub mod circuit_breaker;
pub mod overflow;
pub mod priority;
pub mod shutdown;

// Resilience primitives are implemented in reconnect/heartbeat/limiter/backpressure modules.

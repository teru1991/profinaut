pub mod adapter;
pub mod backpressure;
pub mod connection;
pub mod ext_runtime;
pub mod heartbeat;
pub mod integrity;
pub mod limiter;
pub mod private_runtime;
pub mod public_runtime;
pub mod reconnect;
pub mod session;

// New stabilizers (spec-fixed)
pub mod circuit_breaker;
pub mod overflow;
pub mod priority;
pub mod shutdown;

// Resilience primitives are implemented in reconnect/heartbeat/limiter/backpressure modules.

pub mod backoff;
pub mod restart;
pub mod supervisor;

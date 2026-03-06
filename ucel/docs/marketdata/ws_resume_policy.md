# WS Resume Policy

- Resume candidates are reconstructed from durable states: active/inflight/pending/reconnect/resume-pending.
- Resume directive is derived from failure class + integrity mode.
- Private streams branch to reauth-then-resubscribe.
- Public streams branch to resubscribe or resnapshot-then-resubscribe.

# E: Safety Enforcement Contract

## Non-negotiable
- stop = push (HALT can always be triggered)
- allow = pull (lease)
- lease missing/expired => BLOCK converges (fail-close)
- unknown/missing inputs => fail-close

## Hard enforcement point
- Execution live send MUST be physically enforced at the final send point:
  - E lease ok (TTL=20s, renew=5s)
  - J policy ALLOW
  - audit chain verify ok AND audit append ok
- If any of these fails, live send MUST NOT happen.

## Audit rule
- If audit cannot be written, break-glass/suppress must be rejected.
- HALT remains the highest-priority operation (can be executed even if audit write fails).

-- Minimal local bootstrap for ledger/ops workload.
CREATE SCHEMA IF NOT EXISTS platform AUTHORIZATION CURRENT_USER;

CREATE TABLE IF NOT EXISTS platform.healthcheck (
  id BIGSERIAL PRIMARY KEY,
  checked_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  status TEXT NOT NULL
);

INSERT INTO platform.healthcheck (status)
VALUES ('ok')
ON CONFLICT DO NOTHING;

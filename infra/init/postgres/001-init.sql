-- Bootstrap required local databases, schemas, and users for ledger/serving workloads.

DO $$
BEGIN
   IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'ledger_user') THEN
      CREATE ROLE ledger_user LOGIN PASSWORD 'change-me-ledger';
   END IF;
END
$$;

DO $$
BEGIN
   IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'serving_user') THEN
      CREATE ROLE serving_user LOGIN PASSWORD 'change-me-serving';
   END IF;
END
$$;

SELECT 'CREATE DATABASE ledger'
WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'ledger')\gexec

SELECT 'CREATE DATABASE serving'
WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'serving')\gexec

\connect ledger
CREATE SCHEMA IF NOT EXISTS ledger AUTHORIZATION ledger_user;
GRANT ALL PRIVILEGES ON DATABASE ledger TO ledger_user;
GRANT USAGE, CREATE ON SCHEMA ledger TO ledger_user;
CREATE TABLE IF NOT EXISTS ledger.healthcheck (
  id BIGSERIAL PRIMARY KEY,
  checked_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  status TEXT NOT NULL
);
INSERT INTO ledger.healthcheck (status) VALUES ('ok');

\connect serving
CREATE SCHEMA IF NOT EXISTS serving AUTHORIZATION serving_user;
GRANT ALL PRIVILEGES ON DATABASE serving TO serving_user;
GRANT USAGE, CREATE ON SCHEMA serving TO serving_user;
CREATE TABLE IF NOT EXISTS serving.healthcheck (
  id BIGSERIAL PRIMARY KEY,
  checked_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  status TEXT NOT NULL
);
INSERT INTO serving.healthcheck (status) VALUES ('ok');

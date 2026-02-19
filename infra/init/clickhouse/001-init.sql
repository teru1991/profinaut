CREATE DATABASE IF NOT EXISTS profinaut;

CREATE TABLE IF NOT EXISTS profinaut.healthcheck
(
    checked_at DateTime DEFAULT now(),
    status LowCardinality(String)
)
ENGINE = MergeTree
ORDER BY checked_at;

INSERT INTO profinaut.healthcheck (status) VALUES ('ok');

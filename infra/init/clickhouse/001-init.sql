CREATE DATABASE IF NOT EXISTS profinaut;
CREATE DATABASE IF NOT EXISTS gold;

CREATE TABLE IF NOT EXISTS gold.asset_prices
(
    symbol LowCardinality(String),
    ts DateTime,
    open Float64,
    high Float64,
    low Float64,
    close Float64,
    volume Float64
)
ENGINE = MergeTree
ORDER BY (symbol, ts);

CREATE TABLE IF NOT EXISTS gold.signals
(
    signal_id UUID,
    symbol LowCardinality(String),
    ts DateTime,
    signal_type LowCardinality(String),
    score Float64
)
ENGINE = MergeTree
ORDER BY (symbol, ts, signal_id);

CREATE TABLE IF NOT EXISTS profinaut.healthcheck
(
    checked_at DateTime DEFAULT now(),
    status LowCardinality(String)
)
ENGINE = MergeTree
ORDER BY checked_at;

INSERT INTO profinaut.healthcheck (status) VALUES ('ok');

# ADR-0003: Serving stores = ClickHouse + PostgreSQL + Valkey

- Status: Accepted
- Date: 2026-02-19

## Context
Serving 層は分析系・整合系・キャッシュ系で要件が異なる。

## Decision
- OLAP: ClickHouse（Apache-2.0）
- OLTP: PostgreSQL（PostgreSQL License）
- Cache: Valkey（BSD-3-Clause）
- Redis は採用しない

## Rationale
- 役割分離により性能と整合性要件を両立
- free OSS 制約に適合
- Valkey 標準化で cache 層の運用方針を明確化

## Consequences
- query/API ごとに serving source of truth を明示
- cache miss 時は OLAP/OLTP から再構築する

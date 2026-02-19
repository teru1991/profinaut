# ADR-0001: Lakehouse table format = Apache Iceberg

- Status: Accepted
- Date: 2026-02-19

## Context
Profinaut の Raw-first パイプラインで、Bronze/Silver/Gold の長期運用に耐える table format が必要。

## Decision
Lakehouse table format は Apache Iceberg（Apache-2.0）を採用する。

## Rationale
- OSS ライセンス要件に適合
- schema evolution / snapshot 管理が明確
- Bronze 起点の再計算・監査と相性が良い

## Consequences
- Bronze/Silver/Gold を Iceberg namespace で管理
- snapshot/retention ポリシーを runbook に従って運用

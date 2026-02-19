# ADR-0002: Object storage = SeaweedFS (primary) with RustFS fallback

- Status: Accepted
- Date: 2026-02-19

## Context
S3 互換オブジェクトストレージを free OSS 制約下で標準化する必要がある。

## Decision
- 第一推奨: SeaweedFS（Apache-2.0）
- 代替: RustFS（Apache-2.0）
- MinIO は本 SSOT では採用しない

## Rationale
- ライセンス条件（無料 OSS）の明確適合
- Raw-first Bronze の大容量・長期保持要件に対応
- 代替実装（RustFS）を定義して単一実装依存を回避

## Consequences
- runbook に SeaweedFS -> RustFS failover 手順を持つ
- Bronze URI/RawRef は S3 互換前提で固定

# Coverage Migration Policy (v1 -> v2)

- `coverage/*.yaml` は legacy evidence として保持する。
- Gate 判定の主軸は `coverage_v2/*.yaml`。
- v1/v2 scope drift は SSOT consistency gate で検知し、明示例外なしでは fail する。
- family split venue は canonical bridge ルール（docs/ssot/ssot_consistency_policy.md）で吸収し、将来は canonical を family 単位へ統一する。

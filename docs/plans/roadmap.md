# Roadmap v1.0（計画）

## 目的
- UCELを「横断SDK」として確立し、Collector/Execution/IR/On-chainを契約駆動で積み上げる
- 1人運用で安全に回る（止められる・戻せる・証明できる）
- 将来販売に耐える（Support Bundle / Audit / Replay）

---

## マイルストーン（順序）
### M0: SSOT固定
- docs/contracts を導入し CIでcompile必須
- docs/specs（固定仕様）を整備
- legacy docs を隔離

### M1: Safety・監査・診断（横断基盤）
- SafetyState（HALT/FLATTEN/CLOSE_ONLY/SAFE_MODE）を単一正で運用
- AuditEvent NDJSON（append-only）導入
- Support Bundle（最小構成）導入

### M2: Execution（唯一出口）
- 直叩き経路の封鎖（OrderIntent SSOT）
- idempotency + reconciliation（最小）導入
- Safety強制点がExecutionで成立

### M3: Market Data Collector（Golden Standard）
- SSOT三点セット（coverage/ws_rules/symbols）をFullで整備
- Runtime/Daily Gateで PASS を継続
- Active/Shadow 切替の成立（手順化）

### M4: 拡張ドメイン
- IR Connector（R）
- On-chain Connector（Q）
- Backtest/Forward（N）とReplay（O）の接続強化

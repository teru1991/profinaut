# Backup & Restore Drill Runbook v1.0

## 目的
- "バックアップがある"ではなく"復元できる"を証明する

---

## 対象
- Raw/WAL（保存境界）
- SSOT（docs/contracts, coverage/ws_rules/symbols 等）
- StartupReport / IntegrityReport / Audit NDJSON
- Support Bundle（直近）

---

## 月次演習（推奨）
1) ある日付のデータ（Raw/WAL）を選定
2) 別ディレクトリ/別マシンへ復元
3) リプレイ導線（ReplayPointers）で再現を試行
4) Integrity Report の再生成または照合
5) 演習結果を監査イベントとして残す（MANUAL_OPERATION）

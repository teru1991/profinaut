# contracts/schemas — LEGACY

> **このディレクトリは LEGACY です。新しいスキーマの追加・更新は行わないでください。**

## 状態

| 状態 | 説明 |
|------|------|
| **LEGACY** | このディレクトリのスキーマは旧来の用途向けです。現在も参照されているファイルは保持していますが、メンテナンスは最小限にとどめます。 |

## SSOT（単一信頼源）

契約スキーマの **SSOT は `docs/contracts/`** です。

- 新規スキーマ → `docs/contracts/*.schema.json` に追加してください。
- スキーマの更新 → `docs/contracts/` のファイルを修正し、`schema_version` を上げてください。
- CI 検証 → `.github/workflows/contracts-gate.yml` が `docs/contracts/` 配下の全スキーマを Ajv でコンパイル検証します。

## このディレクトリに残っているファイルについて

以下のファイルは既存コードから参照されているため残していますが、新しい参照を追加してはいけません。

| ファイル | 用途 |
|----------|------|
| `ack.schema.json` | ACK メッセージ |
| `audit.schema.json` | 監査ログ（→ SSOT: `docs/contracts/audit_event.schema.json`） |
| `capabilities.schema.json` | ケイパビリティ宣言 |
| `command.schema.json` | コマンドメッセージ |
| `heartbeat.schema.json` | ハートビート |
| `module.schema.json` | モジュール定義 |
| `module_run.schema.json` | モジュール実行 |
| `policy_decision.schema.json` | ポリシー判定 |
| `reconcile.schema.json` | リコンサイル |
| `safe_mode.schema.json` | セーフモード（→ SSOT: `docs/contracts/safety_state.schema.json`） |

## 移行方針

将来的にはこのディレクトリのファイルを `docs/contracts/` に統合し、このディレクトリを削除します。移行タスクは別 PR で対応します。

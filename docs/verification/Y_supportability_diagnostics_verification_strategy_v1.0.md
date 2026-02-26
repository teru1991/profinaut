分類理由: 検証戦略と回帰・互換性テスト方針を定義する内容のため「検証資料」に分類。

# Y: Supportability / Diagnostics Governance Spec v1.0（SSOT / Level 1）

## 1.5 検証戦略（Verification Strategy）

### 1.5.1 Golden Bundle（回帰資材）
- テスト用bundle（golden）を用意し、analyzer要約が期待通りであることを回帰検証する :contentReference[oaicite:37]{index=37}

### 1.5.2 Redaction回帰
- 疑似JWT/鍵/秘密っぽい文字列を混ぜたログを生成し、以下を自動テストする :contentReference[oaicite:38]{index=38}
  - bundleに残留しない
  - analyzerが漏洩警告を出さない

### 1.5.3 負荷・耐障害
- bundle生成が本番を阻害しない（上限が効く） :contentReference[oaicite:39]{index=39}
- 再起動ループ時に自動bundleが自己DoSしない :contentReference[oaicite:40]{index=40}
- 観測ギャップ発生時でも最小限の診断が残る :contentReference[oaicite:41]{index=41}

### 1.5.4 互換性テスト（diag_semver）
- 旧bundleを新analyzerで読める :contentReference[oaicite:42]{index=42}
- deprecation期間のI/Fが壊れていない :contentReference[oaicite:43]{index=43}

### 1.5.5 Runbook Drift検査
- Runbook内の参照（メトリクス名/コマンド/リンク）が存在するかCIで検査 :contentReference[oaicite:44]{index=44}

TODO:
- 期待値（goldenの期待要約）をどの形式で管理するか、テスト失敗時の差分表示方法を定義する。

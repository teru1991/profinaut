# Key Rotation Runbook v1.0

## 原則
- 秘密はログ/監査/メトリクス/Support Bundleに出さない
- ローテは段階的（新鍵→検証→切替→旧鍵失効）
- 失敗時は安全縮退（SAFE_MODE/HALT）

---

## 手順（概要）
1) 新しい鍵を発行（用途分離：read/trade/withdraw）
2) Secret Providerへ登録（secret_refのみが設定に現れることを確認）
3) Shadow/検証環境で疎通確認
4) 本番へ切替（SafetyをSAFE_MODEにして慎重に）
5) 旧鍵を失効
6) Support Bundleを生成し、証跡（監査）を残す
